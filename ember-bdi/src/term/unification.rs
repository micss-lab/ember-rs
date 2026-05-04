use alloc::collections::btree_map::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use crate::term::{Ground, NonGround, Structure, Term};
use crate::variable::{Variable, VariableId};

type Result<T> = core::result::Result<T, UnificationFailedError>;

#[derive(Debug, Clone, Copy)]
pub struct UnificationFailedError;

impl core::fmt::Display for UnificationFailedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "unification failed")
    }
}

impl core::error::Error for UnificationFailedError {}

pub trait Unify<Rhs = Self> {
    /// Collect individual bindings without recursive verification that the binding is sound.
    /// This essentially builds a list of aliases.
    fn piece_wise_unify<'a>(&'a self, other: &'a Rhs) -> Result<Vec<Binding<'a>>>;

    fn unify(&self, other: &Rhs) -> Result<Bindings> {
        Bindings::build_from_pieces(self.piece_wise_unify(other)?)
    }
}

impl Unify for Term {
    fn piece_wise_unify<'a>(&'a self, other: &'a Term) -> Result<Vec<Binding<'a>>> {
        use Term::*;

        match (self, other) {
            (Number(n1), Number(n2)) => (*n1 == *n2).then_some(Vec::with_capacity(0)),
            (String(s1), String(s2)) => (s1 == s2).then_some(Vec::with_capacity(0)),
            (Variable(NonGround(v)), t) | (t, Variable(NonGround(v))) => {
                return v.piece_wise_unify(t);
            }
            (Structure(s1), Structure(s2)) => return s1.piece_wise_unify(s2),
            _ => None,
        }
        .ok_or(UnificationFailedError)
    }
}

impl Unify for Structure {
    fn piece_wise_unify<'a>(&'a self, other: &'a Structure) -> Result<Vec<Binding<'a>>> {
        if self.functor != other.functor {
            return Err(UnificationFailedError);
        }

        match (&self.arguments, &other.arguments) {
            (Some(args1), Some(args2)) if args1.len() == args2.len() => {
                let mut bindings = Vec::new();

                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    // The ? handles early returns seamlessly
                    bindings.extend(a1.piece_wise_unify(a2)?);
                }

                Some(bindings)
            }
            (None, None) => Some(Vec::with_capacity(0)),
            _ => None,
        }
        .ok_or(UnificationFailedError)
    }
}

impl Unify<Term> for Variable {
    fn piece_wise_unify<'a>(&'a self, other: &'a Term) -> Result<Vec<Binding<'a>>> {
        Ok(vec![Binding::new(&self, other)])
    }
}

pub struct Binding<'a> {
    variable: &'a Variable,
    value: &'a Term,
}

impl<'a> Binding<'a> {
    pub fn new(variable: &'a Variable, value: &'a Term) -> Self {
        Self { variable, value }
    }
}

pub struct Bindings(BTreeMap<VariableId, Option<Term<Ground>>>);

impl Bindings {
    /// Tries to build a unification map of the collected bindings.
    ///
    /// # Implementation
    ///
    /// The function does the following: given a collection of bindings, constructs
    /// partitions of terms based on variable occurences.
    ///
    /// If in a partition multiple non-variables occur that do not unify with eachother, the
    /// map is invalid. If they do unify, then a new iterator of bindings is returned which
    /// will update the partitions and require rechecking all changed variables.
    fn build_from_pieces<'a>(pieces: impl IntoIterator<Item = Binding<'a>>) -> Result<Self> {
        type PartitionId = usize;

        #[derive(Default)]
        struct Partitions<'a> {
            next_id: PartitionId,
            variable_to_partition: BTreeMap<VariableId, PartitionId>,
            partition_to_term: BTreeMap<PartitionId, &'a Term>,
        }

        impl<'a> Partitions<'a> {
            fn get_or_create(&mut self, variable: &Variable) -> PartitionId {
                *self
                    .variable_to_partition
                    .entry(variable.id)
                    .or_insert_with(|| {
                        let id = self.next_id;
                        self.next_id += 1;
                        id
                    })
            }

            fn merge(
                &mut self,
                variable: &'a Variable,
                alias: &'a Variable,
                queue: &mut Vec<Binding<'a>>,
            ) -> Result<()> {
                let pid1 = self.get_or_create(variable);
                let pid2 = self.get_or_create(alias);

                if pid1 != pid2 {
                    // Update all mappings pointing to pid2 to point to pid1
                    for pid in self.variable_to_partition.values_mut() {
                        if *pid == pid2 {
                            *pid = pid1;
                        }
                    }

                    if let Some(t2) = self.partition_to_term.remove(&pid2) {
                        self.add_term(pid1, t2, queue)?;
                    }
                }
                Ok(())
            }

            fn add_term(
                &mut self,
                pid: PartitionId,
                term: &'a Term,
                queue: &mut Vec<Binding<'a>>,
            ) -> Result<()> {
                if let Some(t1) = self.partition_to_term.get(&pid) {
                    queue.extend(t1.piece_wise_unify(term)?);
                } else {
                    self.partition_to_term.insert(pid, term);
                }
                Ok(())
            }
        }

        let mut partitions = Partitions::default();
        let mut queue: Vec<Binding<'a>> = pieces.into_iter().collect();

        while let Some(Binding { variable, value }) = queue.pop() {
            if let Term::Variable(NonGround(alias)) = value {
                partitions.merge(variable, alias, &mut queue)?;
            } else {
                let pid = partitions.get_or_create(variable);
                partitions.add_term(pid, value, &mut queue)?;
            }
        }

        let partition_assignments = match partitions
            .partition_to_term
            .into_iter()
            .map(|(pid, t)| t.clone().try_into_ground().map(|t| (pid, t)))
            .collect::<Option<BTreeMap<PartitionId, Term<Ground>>>>()
        {
            Some(assignments) => assignments,
            None => return Err(UnificationFailedError),
        };

        let bindings = partitions
            .variable_to_partition
            .into_iter()
            .map(|(vid, pid)| (vid, partition_assignments.get(&pid).map(|t| t.clone())))
            .collect();
        Ok(Bindings(bindings))
    }
}

