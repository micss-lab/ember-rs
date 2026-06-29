use ember_util::sync::AtomicU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableId(VarID);

type VarID = u32;

static NEXT_VARIABLE_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub(crate) id: VariableId,
}

impl Variable {
    pub fn new() -> Self {
        let id = NEXT_VARIABLE_ID.get_increment();
        Self { id: VariableId(id) }
    }
}
