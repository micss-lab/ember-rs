use crate::ast::AtomicFormula;

#[derive(Debug, Clone)]
pub(crate) enum SystemAction {}

impl TryFrom<AtomicFormula> for SystemAction {
    type Error = ();

    fn try_from(value: AtomicFormula) -> Result<Self, Self::Error> {
        Err(())
    }
}
