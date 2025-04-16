use chrono::{DateTime, Utc};
pub use filter::MessageFilter;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::agent::Aid;

use super::sl;

mod filter;

// type Encoding = String;

type Ontology = String;

// type Protocol = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub performative: Performative,
    pub sender: Option<Aid>,
    pub receiver: Receiver,
    pub reply_to: Option<Aid>,
    pub ontology: Option<Ontology>,
    pub content: Content,
    // TODO: Implement these.
    // protocol: Option<Protocol>,
    // conversation_id: Option<String>,
    // reply_with: Option<String>,
    // in_reply_to: Option<String>,
    // reply_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageEnvelope {
    pub to: Vec<Aid>,
    pub from: Option<Aid>,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub acl_representation: AclRepresentation,
    pub parameters: BTreeMap<String, bstr::BString>,
    pub message: MessageKind,
}

impl MessageEnvelope {
    pub fn new(to: Aid, message: Message) -> Self {
        Self {
            to: vec![to],
            from: None,
            date: DateTime::<Utc>::MIN_UTC.into(),
            acl_representation: AclRepresentation::BitEfficient,
            parameters: BTreeMap::new(),
            message: MessageKind::Structured(message),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Receiver {
    Single(Aid),
    Multiple(BTreeSet<Aid>),
}

impl From<Aid> for Receiver {
    fn from(aid: Aid) -> Self {
        Self::Single(aid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Performative {
    AcceptProposal,
    Agree,
    Cancel,
    Cfp,
    Confirm,
    Disconfirm,
    Failure,
    Inform,
    InformIf,
    InformRef,
    NotUnderstood,
    Propose,
    QueryIf,
    QueryRef,
    Refuse,
    RejectProposal,
    Request,
    RequestWhen,
    RequestWhenever,
    Subscribe,
    Proxy,
    Ropagate,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Sl(sl::Content),
    Other {
        kind: Option<OtherLanguage>,
        content: String,
    },
}

impl From<sl::Content> for Content {
    fn from(content: sl::Content) -> Self {
        Self::Sl(content)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    Structured(Message),
    // TODO: Support this.
    // Bytes(bstr::BString),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtherLanguage {
    Ccl,
    Kif,
    Rdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclRepresentation {
    BitEfficient,
}
