use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use bstr::BString;
use core::str::FromStr;

use chrono::{DateTime, Utc};

use crate::agent::aid::Aid;

use self::content::fipa_sl::Sl0Content;

pub use self::filter::MessageFilter;

pub mod content;
pub mod filter;
pub mod repr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportMessage {
    /// Stack of envelopes belonging to the message.
    pub envelopes: MessageEnvelopes,
    /// The payload of the transport message containing the Acl message. The payload can still be
    /// encrypted, or otherwise unreadable by the current MTS in which case it is stored as raw bytes.
    pub payload: Payload,
}

/// Stack of message envelopes belonging to an Acl message.
///
/// Envelopes added after the `base` envelope have to be pushed on top of the stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEnvelopes {
    pub base: MessageEnvelope,
    pub others: Vec<MessageEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEnvelope {
    pub to: Vec<Aid>,
    pub from: Option<Aid>,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub acl_representation: AclRepresentation,
    pub other: Option<BTreeMap<String, bstr::BString>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// The parsed Acl message.
    AclMessage(Message),
    /// Bytes representing an Acl message. These could be encrypted or malformed, hence they are
    /// stored as bytes.
    Bytes(BString),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AclRepresentation {
    String,
    BitEfficient,
    Other(String),
}

/// A FIPA Acl Message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub performative: Performative,
    pub receiver: Option<Receiver>,
    pub ontology: Option<String>,
    // NOTE: This parameter is implicitly encoded in `Content`.
    // pub language: String,
    pub other: Option<BTreeMap<String, BString>>,
    pub content: Option<Content>,
    // TODO: Implement these.
    // pub sender: Option<Aid>,
    // pub reply_to: Option<Aid>,
    // pub encoding: Option<Encoding>,
    // pub protocol: Option<Protocol>,
    // pub conversation_id: Option<String>,
    // pub reply_with: Option<String>,
    // pub in_reply_to: Option<String>,
    // pub reply_by: Option<String>,
}

impl Message {
    pub fn into_transport(self) -> TransportMessage {
        let to = match &self.receiver {
            Some(Receiver::Single(aid)) => Vec::from([aid.clone()]),
            Some(Receiver::Multiple(btree_set)) => Vec::from_iter(btree_set.iter().cloned()),
            None => Vec::with_capacity(0),
        };
        let envelope = MessageEnvelope {
            to,
            from: None,
            date: DateTime::<Utc>::MIN_UTC.into(),
            acl_representation: AclRepresentation::BitEfficient,
            other: None,
        };
        TransportMessage {
            envelopes: MessageEnvelopes {
                base: envelope,
                others: Vec::with_capacity(0),
            },
            payload: Payload::AclMessage(self),
        }
    }
}

impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes = repr::string::encode(self);
        f.write_str(core::str::from_utf8(&bytes).map_err(|_| core::fmt::Error)?)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    FipaSl0(Sl0Content),
    Bytes(Vec<u8>),
    Other {
        kind: Option<OtherLanguage>,
        /// Direct bytes (possibly utf-8) from the message not decoded in any way.
        ///
        /// The difference with [`Content::Bytes`] is that the latter is (en/de)coded in
        /// transit.
        content: BString,
    },
}

impl From<Sl0Content> for Content {
    fn from(content: Sl0Content) -> Self {
        Self::FipaSl0(content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtherLanguage {
    Ccl,
    Kif,
    Rdf,
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
            _ => return Err(()),
        })
    }
}
