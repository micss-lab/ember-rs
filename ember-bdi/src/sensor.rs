use alloc::boxed::Box;

use crate::literal::Literal;
use crate::plan::Trigger;

pub use ember_bdi_macros::Percept;

/// Abstraction over anything that can percept the environment of the agent.
pub trait Perceptor {
    type Percept: Percept;

    /// Poll the sensor for any available perceptions.
    fn percept(&mut self) -> Option<Self::Percept>;
}

pub trait Percept: Sized {
    fn into_beliefs(self) -> impl IntoIterator<Item = (Trigger, Literal)>;
}

// Implementation needed for users with an agent that does not percept the environment.
impl Percept for () {
    fn into_beliefs(self) -> impl IntoIterator<Item = (Trigger, Literal)> {
        []
    }
}

/// Wrapper type around [`Perceptors`] for a better internal interface.
///
/// [`Perceptors`]: crate::sensor::Perceptor
pub struct Sensor<'s, Percept>(Box<dyn crate::sensor::Perceptor<Percept = Percept> + 's>);

impl<'s, PP> Sensor<'s, PP>
where
    PP: Percept,
{
    pub fn new<P>(perceptor: P) -> Self
    where
        P: Perceptor<Percept = PP> + 's,
    {
        Self(Box::new(perceptor))
    }

    // NOTE: This cannot be implemented through the trait due to conflicting `From`
    // implementations.
    /// Poll the sensor for any available perceptions.
    pub(crate) fn percept(&mut self) -> Option<PP> {
        self.0.percept()
    }
}

impl<'s, P, PP> From<P> for Sensor<'s, PP>
where
    P: Perceptor<Percept = PP> + 's,
    PP: Percept,
{
    fn from(perceptor: P) -> Self {
        Self::new(perceptor)
    }
}

impl<Percept> core::fmt::Debug for Sensor<'_, Percept> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Sensor").field(&"<perceptor>").finish()
    }
}
