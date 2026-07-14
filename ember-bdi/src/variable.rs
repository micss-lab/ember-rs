use ember_util::sync::AtomicU32;

mod message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableId(VarID);

impl core::fmt::Display for VariableId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type VarID = u32;

static NEXT_VARIABLE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub(crate) id: VariableId,
}

impl Default for Variable {
    fn default() -> Self {
        Self::new()
    }
}

impl Variable {
    pub fn new() -> Self {
        let id = NEXT_VARIABLE_ID.get_increment();
        Self { id: VariableId(id) }
    }
}
