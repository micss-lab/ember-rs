pub use filter::MessageFilter;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use super::sl;

mod filter;

type Aid = String;

type Encoding = String;

type Ontology = String;

type Protocol = String;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MessageEnvelope {
    pub(crate) to: Vec<Aid>,
    pub(crate) from: Option<Aid>,
    pub(crate) date: chrono::DateTime<chrono::FixedOffset>,
    pub(crate) acl_representation: AclRepresentation,
    pub(crate) parameters: BTreeMap<String, bstr::BString>,
    pub(crate) message: bstr::BString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Receiver {
    Single(Aid),
    Multiple(BTreeSet<Aid>),
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
