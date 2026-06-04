use core::marker::PhantomData;

use ember_util::sync::AtomicU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableId(VarID);

type VarID = u32;

static NEXT_VARIABLE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct Variable<T = ()> {
    pub(crate) id: VariableId,
    /// Type of the value storable by the variable. During unification,if the type of the
    /// variable does not match, it will reject the unification attempt.
    type_: (VariableType, PhantomData<T>),
}

impl<T> Variable<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let id = NEXT_VARIABLE_ID.get_increment();
        Self {
            id: VariableId(id),
            type_: (VariableType::Any, PhantomData),
        }
    }
}

impl PartialEq for Variable<()> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.type_.0 == other.type_.0
    }
}

impl Eq for Variable<()> {}

impl PartialOrd for Variable<()> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Variable<()> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VariableType {
    Any,
    String,
    Number,
}
