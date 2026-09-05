use alloc::borrow::Cow;

use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum SendError {
    Generic(#[error(not(source))] Cow<'static, str>),
}
