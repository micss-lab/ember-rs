use alloc::string::String;

use super::sl::{AgentAction, Concept, Constant, Content, ContentElement, Number, Predicate, Term};

#[derive(Debug)]
pub enum DecodeError {
    InvalidType,
    MissingField(&'static str),
    UnknownField(bstr::BString),
    UnknownConcept(bstr::BString),
    InvalidLength(usize),
    Other(Box<dyn core::error::Error>),
}

pub trait DecodeAgentAction: Sized {
    fn from_agent_action(action: AgentAction) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Action(a) => Self::from_agent_action(*a),
            _ => Err(DecodeError::InvalidType),
        }
    }

    fn from_content(content: Content) -> Result<Self, DecodeError> {
        use ContentElement::*;
        let Some(first) = content.0.into_iter().next() else {
            return Err(DecodeError::InvalidLength(0));
        };
        match first {
            AgentAction(a) => Self::from_agent_action(a),
            Predicate(_) => Err(DecodeError::InvalidType),
        }
    }
}

pub trait DecodePredicate: Sized {
    fn from_predicate(predicate: Predicate) -> Result<Self, DecodeError>;

    fn from_content(content: Content) -> Result<Self, DecodeError> {
        let Some(first) = content.0.into_iter().next() else {
            return Err(DecodeError::InvalidLength(0));
        };
        match first {
            ContentElement::Predicate(p) => Self::from_predicate(p),
            ContentElement::AgentAction(_) => Err(DecodeError::InvalidType),
        }
    }
}

pub trait DecodeConcept: Sized {
    fn from_concept(concept: Concept) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Concept(c) => Self::from_concept(c),
            _ => Err(DecodeError::InvalidType),
        }
    }
}

pub trait DecodeConstant: Sized {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Constant(c) => Self::from_constant(c),
            _ => Err(DecodeError::InvalidType),
        }
    }
}

enum CollectionKind {
    Set,
    Seq,
}
pub trait DecodeCollection: Sized {
    fn from_collection(items: &[Term], kind: CollectionKind) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Set(s) => Self::from_collection(&*s, CollectionKind::Set),
            Term::Sequence(s) => Self::from_collection(&*s, CollectionKind::Seq),
            _ => Err(DecodeError::InvalidType),
        }
    }
}

impl DecodeConstant for bstr::BString {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        match constant {
            Constant::String(s) => Ok(s),
            _ => Err(DecodeError::InvalidType),
        }
    }
}

impl DecodeConstant for String {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        use bstr::ByteVec;
        let s = bstr::BString::from_constant(constant)?;
        s.into_string().map_err(|e| DecodeError::Other(e.into()))
    }
}

impl DecodeConstant for i32 {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        match constant {
            Constant::Number(Number::Int(i)) => Ok(i),
            _ => Err(DecodeError::InvalidType),
        }
    }
}

impl DecodeConstant for f32 {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        match constant {
            Constant::Number(Number::Float(f)) => Ok(f),
            _ => Err(DecodeError::InvalidType),
        }
    }
}
