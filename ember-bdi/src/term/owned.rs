use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};

use bstr::BString;

use ember_util::cmp::TotalCmpF32;

use crate::literal::Literal;
use crate::variable::{Variable, VariableId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Term {
    // TODO: Support full arithmetic formulas here.
    Number(TotalCmpF32),
    String(BString),
    Variable(Variable),
    // A fixed-arity list of terms, unified element-wise (see `unification::traits`).
    //
    // TODO: Support Prolog-style partial-list unification via the cons operator (`'.'/2`, i.e.
    // `[Head|Tail]`) so lists of different lengths can unify by binding a tail variable.
    List(Box<[Term]>),
    Literal(Literal),
}

impl Term {
    pub fn is_ground(&self) -> bool {
        use Term::*;
        match self {
            Number(_) | String(_) => true,
            Variable(_) => false,
            List(items) => items.iter().all(Term::is_ground),
            Literal(literal) => literal.is_ground(),
        }
    }

    pub(crate) fn collect_variables(&self, vars: &mut BTreeSet<VariableId>) {
        match self {
            Term::Variable(v) => {
                vars.insert(v.id);
            }
            Term::List(items) => items.iter().for_each(|t| t.collect_variables(vars)),
            Term::Literal(literal) => literal.collect_variables(vars),
            _ => {}
        }
    }
}

impl core::fmt::Display for Term {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Term::Number(n) => write!(f, "{}", n),
            Term::String(s) => write!(f, "{}", s),
            Term::Variable(v) => write!(f, "var_{}", v.id),
            Term::List(ts) => {
                write!(f, "[")?;
                for (i, t) in ts.into_iter().enumerate() {
                    if i == 0 {
                        write!(f, "{}", t)?;
                    } else {
                        write!(f, ", {}", t)?;
                    }
                }
                write!(f, "]")
            }
            Term::Literal(l) => write!(f, "{}", l),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Structure {
    pub functor: Atom,
    pub arguments: Option<Box<[Term]>>,
}

impl Structure {
    pub fn is_ground(&self) -> bool {
        let Structure { arguments, .. } = self;
        arguments
            .as_ref()
            .map(|args| args.iter().all(|a| a.is_ground()))
            .unwrap_or(true)
    }

    pub(crate) fn collect_variables(&self, variables: &mut BTreeSet<VariableId>) {
        if let Some(args) = self.arguments.as_ref() {
            args.iter()
                .for_each(|arg| arg.collect_variables(&mut *variables))
        }
    }

    pub(crate) fn atom_and_arity(&self) -> (Atom, usize) {
        (
            self.functor.clone(),
            self.arguments.as_ref().map(|args| args.len()).unwrap_or(0),
        )
    }
}

impl core::fmt::Display for Structure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.functor.display())?;
        if let Some(args) = self.arguments.as_ref() {
            write!(f, "(")?;
            for (i, arg) in args.into_iter().enumerate() {
                if i == 0 {
                    write!(f, "{}", arg)?;
                } else {
                    write!(f, ", {}", arg)?;
                }
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Atom(pub String);

impl<T> From<T> for Atom
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl Atom {
    /// Returns a displayable representation of the atom.
    ///
    /// Atom cannot directly implement [`Display`] as this would result in conflicting
    /// implementations with the `From` impl above.
    ///
    /// [`Display`]: core::fmt::Display
    pub fn display(&self) -> impl core::fmt::Display {
        &self.0
    }
}
