#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    EmptyFunctor,
    FunctorContainsNull,
    VariableNameEmpty,
    VariableNameContainsNull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    ParseFailed,
}
