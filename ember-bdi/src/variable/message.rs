use alloc::format;

use ember_core::message::content::ember_bdil::Variable as MessageVariable;

use super::Variable;

impl Variable {
    pub(crate) fn into_message_variable(self) -> MessageVariable {
        let Self { id } = self;
        MessageVariable {
            name: format!("Variable_{id}"),
        }
    }
}
