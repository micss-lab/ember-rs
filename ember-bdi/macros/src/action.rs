use std::collections::VecDeque;

use crate::ast::{AtomicFormula, Term};

#[derive(Debug, Clone)]
pub enum BuiltinAction {
    Log(String, Box<[Term]>),
    StopPlatform,
}

impl TryFrom<AtomicFormula> for BuiltinAction {
    type Error = ();

    fn try_from(AtomicFormula { functor, arguments }: AtomicFormula) -> Result<Self, Self::Error> {
        match functor.0.as_str() {
            "log" => Self::parse_log(arguments),
            "stop_platform" => Self::parse_stop_platform(arguments),
            _ => Err(()),
        }
    }
}

impl BuiltinAction {
    fn parse_log(arguments: Option<Box<[Term]>>) -> Result<Self, ()> {
        let mut arguments = VecDeque::from_iter(arguments.unwrap_or_default());

        let Some(level) = arguments.pop_front() else {
            return Err(());
        };
        let level = match level {
            Term::String(s) => s,
            _ => return Err(()),
        };

        Ok(Self::Log(level, Vec::from(arguments).into_boxed_slice()))
    }

    fn parse_stop_platform(arguments: Option<Box<[Term]>>) -> Result<Self, ()> {
        arguments
            .is_none_or(|args| args.is_empty())
            .then_some(Self::StopPlatform)
            .ok_or(())
    }
}
