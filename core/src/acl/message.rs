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
pub(crate) struct Message {
    performative: Performative,
    sender: Option<Aid>,
    receiver: Receiver,
    reply_to: Option<Aid>,
    content: Content,
    // TODO: Implement these.
    // ontology: Option<Ontology>,
    // protocol: Option<Protocol>,
    // conversation_id: Option<String>,
    // reply_with: Option<String>,
    // in_reply_to: Option<String>,
    // reply_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MessageEnvelope {
    to: Vec<Aid>,
    from: Option<Aid>,
    date: chrono::DateTime<chrono::FixedOffset>,
    acl_representation: AclRepresentation,
    parameters: BTreeMap<String, bstr::BString>,
    message: bstr::BString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Receiver {
    Single(Aid),
    Multiple(BTreeSet<Aid>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Performative {
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
enum Content {
    Sl(sl::Content),
    Other {
        kind: Option<OtherLanguage>,
        content: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OtherLanguage {
    Ccl,
    Kif,
    Rdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AclRepresentation {
    BitEfficient,
}
