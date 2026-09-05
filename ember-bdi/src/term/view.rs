use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;

use bstr::{BStr, BString};
use ember_util::cmp::TotalCmpF32;

use crate::literal::{Literal, LiteralView};
use crate::variable::Variable;

use super::{Atom, Structure, Term};

#[derive(Debug)]
pub enum TermView<'a> {
    Term(&'a Term),
    Number(TotalCmpF32),
    String(ViewString<'a>),
    Variable(Variable),
    List(Rc<[TermView<'a>]>),
    Literal(LiteralView<'a>),
}

impl Clone for TermView<'_> {
    fn clone(&self) -> Self {
        match self {
            Self::Term(term) => Self::Term(term),
            Self::Number(n) => Self::Number(*n),
            Self::String(s) => Self::String(s.clone()),
            Self::Variable(v) => Self::Variable(v.clone()),
            Self::List(items) => Self::List(items.clone()),
            Self::Literal(literal) => Self::Literal(literal.clone()),
        }
    }
}

impl PartialEq for TermView<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TermView::Number(a), TermView::Number(b)) => a == b,
            (TermView::String(a), TermView::String(b)) => a == b,
            (TermView::Variable(a), TermView::Variable(b)) => a == b,
            (TermView::List(a), TermView::List(b)) => a.len() == b.len() && a.iter().eq(b.iter()),
            (TermView::Literal(a), TermView::Literal(b)) => a == b,
            (TermView::Term(t), other) | (other, TermView::Term(t)) => {
                eq_cmp::term_eq_view(t, other)
            }
            _ => false,
        }
    }
}

impl Eq for TermView<'_> {}

impl PartialOrd for TermView<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TermView<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (TermView::Number(a), TermView::Number(b)) => a.cmp(b),
            (TermView::String(a), TermView::String(b)) => a.cmp(b),
            (TermView::Variable(a), TermView::Variable(b)) => a.cmp(b),
            (TermView::List(a), TermView::List(b)) => a.iter().cmp(b.iter()),
            (TermView::Literal(a), TermView::Literal(b)) => a.cmp(b),
            (TermView::Term(t), _) => eq_cmp::term_cmp_view(t, other),
            (_, TermView::Term(t)) => eq_cmp::term_cmp_view(t, self).reverse(),
            _ => eq_cmp::rank(self).cmp(&eq_cmp::rank(other)),
        }
    }
}

impl<'a> From<&'a Term> for TermView<'a> {
    fn from(value: &'a Term) -> Self {
        match value {
            Term::Number(n) => TermView::Number(*n),
            t => TermView::Term(t),
        }
    }
}

impl From<Term> for TermView<'static> {
    fn from(term: Term) -> Self {
        match term {
            Term::Number(n) => TermView::Number(n),
            Term::String(s) => TermView::String(s.into()),
            Term::Variable(v) => TermView::Variable(v.clone()),
            Term::List(terms) => TermView::List(terms.into_iter().map(Into::into).collect()),
            Term::Literal(literal) => TermView::Literal(literal.into()),
        }
    }
}

impl<'a> From<&'a Literal> for TermView<'a> {
    fn from(literal: &'a Literal) -> Self {
        TermView::Literal(literal.into())
    }
}

impl From<Literal> for LiteralView<'static> {
    fn from(Literal { negated, structure }: Literal) -> Self {
        LiteralView {
            negated,
            structure: structure.into(),
        }
    }
}

impl<'a> TermView<'a> {
    pub(crate) fn as_variable(&self) -> Option<&'a Variable> {
        let Self::Term(term) = self else {
            return None;
        };
        let Term::Variable(v) = term else {
            return None;
        };
        Some(v)
    }
}

impl TermView<'_> {
    pub(crate) fn to_owned(&self) -> Term {
        match self {
            TermView::Term(term) => (*term).clone(),
            TermView::Number(n) => Term::Number(*n),
            TermView::String(s) => Term::String(s.as_bstr().to_owned()),
            TermView::Variable(v) => Term::Variable(v.clone()),
            TermView::List(items) => Term::List(
                items
                    .iter()
                    .map(TermView::to_owned)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            TermView::Literal(literal) => Term::Literal(literal.to_owned()),
        }
    }

    pub(crate) fn to_owned_view(&self) -> TermView<'static> {
        match self {
            TermView::Term(term) => (*term).clone().into(),
            TermView::Number(n) => TermView::Number(*n),
            TermView::String(s) => TermView::String(s.to_owned_view()),
            TermView::Variable(v) => TermView::Variable(v.clone()),
            TermView::List(terms) => {
                TermView::List(terms.iter().map(|t| t.to_owned_view()).collect())
            }
            TermView::Literal(l) => TermView::Literal(l.to_owned_view()),
        }
    }
}

#[cfg(test)]
impl Term {
    pub(crate) fn as_view(&self) -> TermView<'_> {
        self.into()
    }
}

/// A possibly-owned string cheaply clonable in both the borrowed and owned case.
#[derive(Debug, Clone)]
pub enum ViewString<'a> {
    Borrowed(&'a BStr),
    Owned(Rc<BStr>),
}

impl ViewString<'_> {
    pub(crate) fn as_bstr(&self) -> &BStr {
        match self {
            ViewString::Borrowed(s) => s,
            ViewString::Owned(s) => s,
        }
    }

    pub(crate) fn to_owned_view(&self) -> ViewString<'static> {
        match self {
            ViewString::Borrowed(bstr) => {
                // While wasteful, going through a BString is the best option here
                // without using "unsafe". The `bstr` crate does not provide the rc
                // conversion, nor does it expose enough internals to make this
                // conversion a single safe step.
                ViewString::from((*bstr).to_owned())
            }
            ViewString::Owned(bstr) => ViewString::Owned(bstr.clone()),
        }
    }
}

impl PartialEq for ViewString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_bstr() == other.as_bstr()
    }
}

impl Eq for ViewString<'_> {}

impl PartialOrd for ViewString<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ViewString<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_bstr().cmp(other.as_bstr())
    }
}

impl<'a> From<&'a BStr> for ViewString<'a> {
    fn from(value: &'a BStr) -> Self {
        ViewString::Borrowed(value)
    }
}

impl From<BString> for ViewString<'_> {
    fn from(value: BString) -> Self {
        let bytes: Box<[u8]> = Vec::from(value).into_boxed_slice();
        let bstr: Box<BStr> = bytes.into();
        ViewString::Owned(bstr.into())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructureView<'a> {
    pub functor: Rc<Atom>,
    pub arguments: Option<Rc<[TermView<'a>]>>,
}

impl Clone for StructureView<'_> {
    fn clone(&self) -> Self {
        Self {
            functor: self.functor.clone(),
            arguments: self.arguments.clone(),
        }
    }
}

impl<'a> From<&'a Structure> for StructureView<'a> {
    fn from(Structure { functor, arguments }: &'a Structure) -> Self {
        Self {
            functor: Rc::new(functor.clone()),
            arguments: arguments
                .as_ref()
                .map(|a| a.iter().map(Into::into).collect::<Vec<_>>().into()),
        }
    }
}

impl From<Structure> for StructureView<'static> {
    fn from(Structure { functor, arguments }: Structure) -> Self {
        Self {
            functor: Rc::new(functor),
            arguments: arguments.map(|args| args.into_iter().map(Into::into).collect()),
        }
    }
}

impl<'a> From<&'a Structure> for TermView<'a> {
    fn from(structure: &'a Structure) -> Self {
        Self::Literal(structure.into())
    }
}

impl StructureView<'_> {
    pub(crate) fn to_owned(&self) -> Structure {
        Structure {
            functor: (*self.functor).clone(),
            arguments: self.arguments.as_ref().map(|ts| {
                ts.iter()
                    .map(TermView::to_owned)
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            }),
        }
    }

    pub(crate) fn to_owned_view(&self) -> StructureView<'static> {
        let Self { functor, arguments } = self;
        StructureView {
            functor: functor.clone(),
            arguments: arguments
                .as_ref()
                .map(|args| args.iter().map(|t| t.to_owned_view()).collect()),
        }
    }
}

mod eq_cmp {
    use bstr::ByteSlice;

    use crate::literal::{Literal, LiteralView};
    use crate::term::{Structure, Term};

    use super::{StructureView, TermView};

    /// Rank terms and view-specific types on the same level if they represent the same value.
    pub(super) fn rank(view: &TermView<'_>) -> u8 {
        match view {
            TermView::Term(t) => rank_term(t),
            TermView::Number(_) => 0,
            TermView::String(_) => 1,
            TermView::Variable(_) => 2,
            TermView::List(_) => 3,
            TermView::Literal(_) => 4,
        }
    }

    fn rank_term(term: &Term) -> u8 {
        match term {
            Term::Number(_) => 0,
            Term::String(_) => 1,
            Term::Variable(_) => 2,
            Term::List(_) => 3,
            Term::Literal(_) => 4,
        }
    }

    pub(super) fn term_eq_view(term: &Term, view: &TermView<'_>) -> bool {
        match (term, view) {
            (Term::Number(a), TermView::Number(b)) => a == b,
            (Term::String(a), TermView::String(b)) => a.as_bstr() == b.as_bstr(),
            (Term::Variable(a), TermView::Variable(b)) => a == b,
            (Term::List(a), TermView::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| term_eq_view(x, y))
            }
            (Term::Literal(a), TermView::Literal(b)) => literal_eq_view(a, b),
            (a, TermView::Term(b)) => a == *b,
            _ => false,
        }
    }

    pub(super) fn term_cmp_view(term: &Term, view: &TermView<'_>) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        match (term, view) {
            (Term::Number(a), TermView::Number(b)) => a.cmp(b),
            (Term::String(a), TermView::String(b)) => a.as_bstr().cmp(b.as_bstr()),
            (Term::Variable(a), TermView::Variable(b)) => a.cmp(b),
            (Term::List(a), TermView::List(b)) => {
                let mut a = a.iter();
                let mut b = b.iter();
                loop {
                    return match (a.next(), b.next()) {
                        (Some(x), Some(y)) => match term_cmp_view(x, y) {
                            Ordering::Equal => continue,
                            ord => ord,
                        },
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Less,
                        (Some(_), None) => Ordering::Greater,
                    };
                }
            }
            (Term::Literal(a), TermView::Literal(b)) => literal_cmp_view(a, b),
            (a, TermView::Term(b)) => a.cmp(*b),
            _ => rank_term(term).cmp(&rank(view)),
        }
    }

    fn literal_eq_view(literal: &Literal, view: &LiteralView<'_>) -> bool {
        literal.negated == view.negated && structure_eq_view(&literal.structure, &view.structure)
    }

    fn literal_cmp_view(literal: &Literal, view: &LiteralView<'_>) -> core::cmp::Ordering {
        literal
            .negated
            .cmp(&view.negated)
            .then_with(|| structure_cmp_view(&literal.structure, &view.structure))
    }

    fn structure_eq_view(structure: &Structure, view: &StructureView<'_>) -> bool {
        structure.functor == *view.functor
            && match (&structure.arguments, &view.arguments) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| term_eq_view(x, y))
                }
                _ => false,
            }
    }

    fn structure_cmp_view(structure: &Structure, view: &StructureView<'_>) -> core::cmp::Ordering {
        use core::cmp::Ordering;
        structure.functor.cmp(&view.functor).then_with(|| {
            match (&structure.arguments, &view.arguments) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a), Some(b)) => {
                    let mut a = a.iter();
                    let mut b = b.iter();
                    loop {
                        return match (a.next(), b.next()) {
                            (Some(x), Some(y)) => match term_cmp_view(x, y) {
                                Ordering::Equal => continue,
                                ord => ord,
                            },
                            (None, None) => Ordering::Equal,
                            (None, Some(_)) => Ordering::Less,
                            (Some(_), None) => Ordering::Greater,
                        };
                    }
                }
            }
        })
    }
}
