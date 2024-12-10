use core::ops::{Deref, DerefMut};

pub struct State<R, P> {
    pub(crate) root: R,
    pub(crate) parent: P,
}

impl<R, P> State<R, P> {
    pub fn parent(&mut self) -> &mut P {
        &mut self.parent
    }

    #[doc(hidden)]
    pub fn cut_root(self) -> (R, P) {
        (self.root, self.parent)
    }

    #[doc(hidden)]
    pub fn new(root: R, parent: P) -> State<R, P> {
        Self { root, parent }
    }
}

impl<R, P> Deref for State<R, P> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl<R, P> DerefMut for State<R, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}
