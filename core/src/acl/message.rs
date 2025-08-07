pub use self::filter::MessageFilter;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::str::FromStr;

use bstr::BStr;
use chrono::{DateTime, Utc};

use serde::ser::{SerializeSeq, SerializeStruct};

use crate::acl::ser;
use crate::agent::Aid;

use super::sl;

mod filter;
mod parser;

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
    Propagate,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Sl(sl::Content),
    Bytes(Vec<u8>),
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

impl core::fmt::Display for OtherLanguage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use OtherLanguage::*;
        f.write_str(match self {
            Ccl => "fipa-ccl",
            Kif => "fipa-kif",
            Rdf => "fipa-rdf",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AclRepresentation {
    BitEfficient,
}

impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&ser::to_string(&self))
    }
}

impl Message {
    pub(crate) fn try_from_bytes(bytes: impl AsRef<BStr>) -> Result<Self, ()> {
        let bytes = bytes.as_ref();
        self::parser::messsage::message(&crate::util::parsing::BStr::from(bytes)).map_err(|e| {
            log::error!("error parsing acl message: {}", e);
            log::debug!("{}", bytes);
        })
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

impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Add other fields here.
        let mut message = serializer.serialize_struct(self.performative.as_str(), 3)?;
        // message.serialize_field("sender", &self.sender)?;
        message.serialize_field("receiver", &self.receiver)?;
        match &self.content {
            Content::Sl(sl) => {
                message.serialize_field("lanuage", "fipa-sl0")?;
                message.serialize_field("content", &format!("\"{}\"", sl))?;
            }
            Content::Bytes(b) => {
                message.serialize_field("language", "bytes")?;
                // TODO: Encode as regular bytes when parsing support for acl messages is expanded.
                message
                    .serialize_field("content", &format!("\"{}\"", hex::encode(b.as_slice())))?;
            }
            Content::Other { kind, content } => {
                if let Some(kind) = kind {
                    message.serialize_field("language", &kind.to_string())?;
                }
                message.serialize_field("content", &format!("\"{}\"", content))?;
            }
        }
        message.serialize_field("ontology", &self.ontology)?;
        message.end()
    }
}

impl serde::Serialize for Receiver {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Receiver::Single(r) => r.serialize(serializer),
            Receiver::Multiple(receivers) => {
                let mut rs = serializer.serialize_seq(Some(receivers.len()))?;
                for r in receivers {
                    rs.serialize_element(r)?;
                }
                rs.end()
            }
        }
    }
}
