use alloc::borrow::Cow;
use alloc::collections::BTreeMap;

use crate::term::Term;
use crate::variable::VariableId;

#[derive(Debug)]
pub struct Bindings<'a>(pub(crate) BTreeMap<VariableId, Option<Cow<'a, Term>>>);
