use alloc::boxed::Box;

/// Abstraction over anything that can percept the environment of the agent.
pub trait Perceptor {
    /// The result of percepting the environment.
    type Percept;

    /// Poll the sensor for any available perceptions.
    fn percept(&mut self) -> Option<Self::Percept>;
}

/// Wrapper type around [`Perceptors`] for a better internal interface.
///
/// [`Perceptors`]: crate::sensor::Perceptor
pub struct Sensor<'s, P>(Box<dyn crate::sensor::Perceptor<Percept = P> + 's>);

impl<'s, P> Sensor<'s, P> {
    pub fn new<PP>(perceptor: PP) -> Self
    where
        PP: Perceptor<Percept = P> + 's,
    {
        Self(Box::new(perceptor))
    }

    // NOTE: This cannot be implemented through the trait due to conflicting `From`
    // implementations.
    /// Poll the sensor for any available perceptions.
    pub(crate) fn percept(&mut self) -> Option<P> {
        self.0.percept()
    }
}

impl<'s, P, PP> From<PP> for Sensor<'s, P>
where
    PP: Perceptor<Percept = P> + 's,
{
    fn from(perceptor: PP) -> Self {
        Self::new(perceptor)
    }
}

impl<P> core::fmt::Debug for Sensor<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Sensor").field(&"<perceptor>").finish()
    }
}
