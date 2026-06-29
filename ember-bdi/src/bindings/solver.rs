use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::bindings::{AliasMap, Bindings, StructureView, TermView};
use crate::term::Term;
use crate::unification::constraint::BindingConstraint;
use crate::unification::error::{Result, UnificationError};
use crate::unification::traits::UnifyView;
use crate::variable::VariableId;

pub(super) struct ConstraintSolver<'a> {
    classes: EquivalenceClasses<'a>,
    queue: Vec<BindingConstraint<'a>>,
}

impl<'a> ConstraintSolver<'a> {
    pub(super) fn new(constraints: impl IntoIterator<Item = BindingConstraint<'a>>) -> Self {
        Self {
            classes: EquivalenceClasses::default(),
            queue: constraints.into_iter().collect(),
        }
    }

    pub(super) fn load_existing_bindings(&mut self, existing: &Bindings<'a>) -> Result<()> {
        if let Some(bindings) = &existing.bindings {
            self.register_constraints(
                bindings
                    .iter()
                    .filter_map(|(v, t)| t.as_ref().map(|t| (*v, t.clone()))),
            )?;
        }

        self.register_aliases(existing.aliases.iter().copied())
    }

    pub(super) fn register_constraints<C>(
        &mut self,
        constraints: impl IntoIterator<Item = C>,
    ) -> Result<()>
    where
        C: Into<BindingConstraint<'a>>,
    {
        constraints
            .into_iter()
            .map(|c| c.into())
            .try_for_each(|c| self.classes.register(c, &mut self.queue))
    }

    pub(super) fn register_aliases(
        &mut self,
        aliases: impl IntoIterator<Item = (VariableId, VariableId)>,
    ) -> Result<()> {
        aliases
            .into_iter()
            .try_for_each(|(var1, var2)| self.classes.merge(var1, var2, &mut self.queue))
    }

    pub(super) fn solve(mut self) -> Result<Bindings<'a>> {
        self.process_constraints()?;
        self.finalize()
    }

    fn process_constraints(&mut self) -> Result<()> {
        while let Some(constraint) = self.queue.pop() {
            self.classes.register(constraint, &mut self.queue)?;
        }
        Ok(())
    }

    fn finalize(&mut self) -> Result<Bindings<'a>> {
        let variables: Vec<VariableId> = self.classes.parent.keys().copied().collect();
        for &var in &variables {
            self.classes.find_root(var);
        }

        let mut root_assignments = BTreeMap::new();
        for (&root, term) in &self.classes.root_to_term {
            let resolved = self.classes.resolve_term(term.clone(), &mut Vec::new())?;
            root_assignments.insert(root, resolved);
        }

        let mut bindings = BTreeMap::new();
        let mut root_to_vars: BTreeMap<VariableId, Vec<VariableId>> = BTreeMap::new();

        for &var in &variables {
            let root = self.classes.find_root(var);
            bindings.insert(var, root_assignments.get(&root).cloned());
            root_to_vars.entry(root).or_default().push(var);
        }

        let aliases = self.extract_aliases(root_to_vars);

        Ok(Bindings::new(bindings, aliases))
    }

    fn extract_aliases(&self, root_to_vars: BTreeMap<VariableId, Vec<VariableId>>) -> AliasMap {
        let mut aliases_pairs = Vec::new();
        for vars in root_to_vars.values() {
            if let Some((&first, rest)) = vars.split_first() {
                for &other in rest {
                    aliases_pairs.push((first, other));
                }
            }
        }
        AliasMap::new(aliases_pairs)
    }
}

#[derive(Debug, Default)]
struct EquivalenceClasses<'a> {
    parent: BTreeMap<VariableId, VariableId>,
    root_to_term: BTreeMap<VariableId, TermView<'a>>,
}

impl<'a> EquivalenceClasses<'a> {
    fn find_root(&mut self, var: VariableId) -> VariableId {
        let p = *self.parent.entry(var).or_insert(var);
        if p != var {
            let root = self.find_root(p);
            self.parent.insert(var, root);
            root
        } else {
            p
        }
    }

    fn root_of(&self, var: VariableId) -> Option<VariableId> {
        let mut current = var;
        loop {
            match self.parent.get(&current) {
                Some(&p) if p == current => return Some(current),
                Some(&p) => current = p,
                None => return None,
            }
        }
    }

    fn merge(
        &mut self,
        var1: VariableId,
        var2: VariableId,
        queue: &mut Vec<BindingConstraint<'a>>,
    ) -> Result<()> {
        let root1 = self.find_root(var1);
        let root2 = self.find_root(var2);

        if root1 != root2 {
            self.parent.insert(root2, root1);

            if let Some(t2) = self.root_to_term.remove(&root2) {
                self.add_term(root1, t2, queue)?;
            }
        }

        Ok(())
    }

    fn register(
        &mut self,
        BindingConstraint { variable, value }: BindingConstraint<'a>,
        queue: &mut Vec<BindingConstraint<'a>>,
    ) -> Result<()> {
        if let Some(alias) = value.as_variable() {
            self.merge(variable, alias.id, queue)
        } else {
            let root = self.find_root(variable);
            self.add_term(root, value, queue)
        }
    }

    fn add_term(
        &mut self,
        root: VariableId,
        term: TermView<'a>,
        queue: &mut Vec<BindingConstraint<'a>>,
    ) -> Result<()> {
        if let Some(t1) = self.root_to_term.get(&root) {
            queue.extend(t1.clone().collect_constraints(term)?);
        } else {
            self.root_to_term.insert(root, term);
        }
        Ok(())
    }

    fn resolve_term(
        &self,
        term: TermView<'a>,
        visiting: &mut Vec<VariableId>,
    ) -> Result<TermView<'a>> {
        match term {
            TermView::Term(term) => match term {
                Term::Number(n) => Ok(TermView::Number(*n)),
                Term::String(_) => Ok(TermView::Term(term)),

                Term::Variable(v) => {
                    let Some(root) = self.root_of(v.id) else {
                        return Ok(TermView::Term(term));
                    };
                    Ok(self
                        .resolve_root(root, visiting)?
                        .unwrap_or(TermView::Term(term)))
                }
                Term::Literal {
                    negated: n,
                    structure: s,
                } => {
                    let args = match &s.arguments {
                        Some(args) => {
                            let mut resolved_args = Vec::with_capacity(args.len());
                            for arg in args.iter() {
                                resolved_args
                                    .push(self.resolve_term(TermView::Term(arg), visiting)?);
                            }
                            Some(resolved_args.into_boxed_slice())
                        }
                        None => None,
                    };
                    Ok(TermView::Literal {
                        negated: *n,
                        structure: StructureView {
                            functor: &s.functor,
                            arguments: args,
                        },
                    })
                }
            },
            TermView::Literal { negated, structure } => {
                let args = match structure.arguments {
                    Some(args) => {
                        let mut resolved_args = Vec::with_capacity(args.len());
                        for arg in args {
                            resolved_args.push(self.resolve_term(arg, visiting)?);
                        }
                        Some(resolved_args.into_boxed_slice())
                    }
                    None => None,
                };
                Ok(TermView::Literal {
                    negated,
                    structure: StructureView {
                        functor: structure.functor,
                        arguments: args,
                    },
                })
            }
            TermView::Number(_) => Ok(term.clone()),

            TermView::Variable(v) => {
                let Some(root) = self.root_of(v.id) else {
                    return Ok(TermView::Variable(v));
                };
                Ok(self
                    .resolve_root(root, visiting)?
                    .unwrap_or(TermView::Variable(v)))
            }
        }
    }

    fn resolve_root(
        &self,
        root: VariableId,
        visiting: &mut Vec<VariableId>,
    ) -> Result<Option<TermView<'a>>> {
        if visiting.contains(&root) {
            return Err(UnificationError::CyclicReference);
        }
        let Some(term) = self.root_to_term.get(&root) else {
            return Ok(None);
        };

        visiting.push(root);
        let result = self.resolve_term(term.clone(), visiting);
        visiting.pop();
        result.map(Some)
    }
}
