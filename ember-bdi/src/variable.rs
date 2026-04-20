use core::marker::PhantomData;

type VarID = u32;

pub struct Variable<T = ()> {
    id: VarID,
    /// Type of the value storable by the variable. During unification,if the type of the
    /// variable does not match, it will reject the unification attempt.
    type_: (VariableType, PhantomData<T>),
}

enum VariableType {
    String,
    Number,
}
