use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::sl::{
    AgentAction, Concept, Constant, Content, ContentElement, Number, Predicate, Seq, Set, Term,
};

#[derive(Debug)]
pub enum DecodeError {
    InvalidType,
    MissingField(&'static str),
    UnknownField(bstr::BString),
    UnknownConcept(bstr::BString),
    InvalidLength(usize),
    Other(Box<dyn core::error::Error>),
}

pub trait AgentActionCodec: Sized {
    // From

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

    // Into

    fn into_agent_action(self) -> AgentAction;

    fn into_content_element(self) -> ContentElement {
        ContentElement::AgentAction(self.into_agent_action())
    }

    fn into_content(self) -> Content {
        Content(vec![self.into_content_element()])
    }
}

pub trait PredicateCodec: Sized {
    // From

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

    // Into

    fn into_predicate(self) -> Predicate;

    fn into_content_element(self) -> ContentElement {
        ContentElement::Predicate(self.into_predicate())
    }

    fn into_content(self) -> Content {
        Content(vec![self.into_content_element()])
    }
}

pub trait ConceptCodec: Sized {
    // From

    fn from_concept(concept: Concept) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Concept(c) => Self::from_concept(c),
            _ => Err(DecodeError::InvalidType),
        }
    }

    // Into

    fn into_concept(self) -> Concept;

    fn into_term(self) -> Term {
        Term::Concept(self.into_concept())
    }
}

pub trait ConstantCodec: Sized {
    // From

    fn from_constant(constant: Constant) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Constant(c) => Self::from_constant(c),
            _ => Err(DecodeError::InvalidType),
        }
    }

    // Into

    fn into_constant(self) -> Constant;

    fn into_term(self) -> Term {
        Term::Constant(self.into_constant())
    }
}

pub enum CollectionKind {
    Set,
    Seq,
}
pub trait CollectionCodec: Sized {
    // From

    fn from_collection(items: &[Term], kind: CollectionKind) -> Result<Self, DecodeError>;

    fn from_term(term: Term) -> Result<Self, DecodeError> {
        match term {
            Term::Set(s) => Self::from_collection(&s, CollectionKind::Set),
            Term::Sequence(s) => Self::from_collection(&s, CollectionKind::Seq),
            _ => Err(DecodeError::InvalidType),
        }
    }

    // Into

    fn into_collection(self) -> (CollectionKind, impl IntoIterator<Item = Term>);

    fn into_term(self) -> Term {
        let (kind, iter) = self.into_collection();
        match kind {
            CollectionKind::Set => Term::Set(Set::from_iter(iter)),
            CollectionKind::Seq => Term::Sequence(Seq::from_iter(iter)),
        }
    }
}

impl AgentActionCodec for AgentAction {
    fn from_agent_action(action: AgentAction) -> Result<Self, DecodeError> {
        Ok(action)
    }

    fn into_agent_action(self) -> AgentAction {
        self
    }
}

impl ConceptCodec for Concept {
    fn from_concept(concept: Concept) -> Result<Self, DecodeError> {
        Ok(concept)
    }

    fn into_concept(self) -> Concept {
        self
    }
}

impl ConstantCodec for bstr::BString {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        match constant {
            Constant::String(s) => Ok(s),
            _ => Err(DecodeError::InvalidType),
        }
    }

    fn into_constant(self) -> Constant {
        Constant::String(self)
    }
}

impl ConstantCodec for String {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        use bstr::ByteVec;
        let s = bstr::BString::from_constant(constant)?;
        Vec::from(s)
            .into_string()
            .map_err(|e| DecodeError::Other(e.to_string().into()))
    }

    fn into_constant(self) -> Constant {
        Constant::String(self.into())
    }
}

impl ConstantCodec for i32 {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        match constant {
            Constant::Number(Number::Int(i)) => Ok(i),
            _ => Err(DecodeError::InvalidType),
        }
    }

    fn into_constant(self) -> Constant {
        Constant::Number(Number::Int(self))
    }
}

impl ConstantCodec for f32 {
    fn from_constant(constant: Constant) -> Result<Self, DecodeError> {
        match constant {
            Constant::Number(Number::Float(f)) => Ok(f),
            _ => Err(DecodeError::InvalidType),
        }
    }

    fn into_constant(self) -> Constant {
        Constant::Number(Number::Float(self))
    }
}
