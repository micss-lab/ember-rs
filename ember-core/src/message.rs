use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::str::FromStr;

use chrono::{DateTime, Utc};

use crate::agent::aid::Aid;

pub use self::filter::MessageFilter;

pub mod content;
pub mod filter;
pub mod repr;

mod ser;

#[derive(Debug, Clone, PartialEq)]
pub struct MessageEnvelope {
    pub to: Vec<Aid>,
    pub from: Option<Aid>,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub acl_representation: AclRepresentation,
    pub parameters: BTreeMap<String, bstr::BString>,
    pub message: MessageKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub performative: Performative,
    pub sender: Option<Aid>,
    pub receiver: Receiver,
    pub reply_to: Option<Aid>,
    pub ontology: Option<String>,
    pub content: Content,
    // TODO: Implement these.
    // protocol: Option<Protocol>,
    // conversation_id: Option<String>,
    // reply_with: Option<String>,
    // in_reply_to: Option<String>,
    // reply_by: Option<String>,
}

impl MessageEnvelope {
    pub fn new(to: Aid, message: Message) -> Self {
        Self {
            to: vec![to],
            from: None,
            date: DateTime::<Utc>::MIN_UTC.into(),
            acl_representation: AclRepresentation::BitEfficient,
            parameters: BTreeMap::new(),
            message: MessageKind::Parsed(message),
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
    Propagate,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Structured(self::content::Content),
    Bytes(Vec<u8>),
    Other {
        kind: Option<OtherLanguage>,
        content: String,
    },
}

impl From<self::content::Content> for Content {
    fn from(content: self::content::Content) -> Self {
        Self::Structured(content)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    Parsed(Message),
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

// TODO: Remove the need for these by using serialize directly.
mod todo {
    use super::Message;

    impl core::fmt::Display for Message {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str(&super::repr::human::to_string(&self))
        }
    }

    impl Message {
        #[allow(unused)]
        pub fn try_from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, ()> {
            super::repr::human::try_from_bytes(bytes)
        }
    }
}

impl Performative {
    fn as_str(&self) -> &'static str {
        use Performative::*;
        match self {
            AcceptProposal => "accept-proposal",
            Agree => "agree",
            Cancel => "cancel",
            Cfp => "cfp",
            Confirm => "confirm",
            Disconfirm => "disconfirm",
            Failure => "failure",
            Inform => "inform",
            InformIf => "inform-if",
            InformRef => "inform-ref",
            NotUnderstood => "not-understood",
            Propose => "propose",
            QueryIf => "query-if",
            QueryRef => "query-ref",
            Refuse => "refuse",
            RejectProposal => "reject-proposal",
            Request => "request",
            RequestWhen => "request-when",
            RequestWhenever => "request-whenever",
            Subscribe => "subscribe",
            Proxy => "proxy",
            Propagate => "propagate",
            Unknown => "unknown",
        }
    }
}

impl FromStr for Performative {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Performative::*;
        Ok(match s {
            "accept-proposal" => AcceptProposal,
            "agree" => Agree,
            "cancel" => Cancel,
            "cfp" => Cfp,
            "confirm" => Confirm,
            "disconfirm" => Disconfirm,
            "failure" => Failure,
            "inform" => Inform,
            "inform-if" => InformIf,
            "inform-ref" => InformRef,
            "not-understood" => NotUnderstood,
            "propose" => Propose,
            "query-if" => QueryIf,
            "query-ref" => QueryRef,
            "refuse" => Refuse,
            "reject-proposal" => RejectProposal,
            "request" => Request,
            "request-when" => RequestWhen,
            "request-whenever" => RequestWhenever,
            "subscribe" => Subscribe,
            "proxy" => Proxy,
            "propagate" => Propagate,

            // TODO: Should the error case become the unknown performative?
            "unknown" => Unknown,
            _ => return Err(()),
        })
    }
}
