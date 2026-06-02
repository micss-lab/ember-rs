use alloc::borrow::Cow;

use crate::environment::Environment;

pub use self::aid::Aid;

pub mod aid;

pub trait Agent {
    fn update(&mut self, environment: &mut Environment) -> bool;

    fn get_name(&self) -> Cow<str>;
}
