use crate::message::Content;
use crate::message::{Message, OtherLanguage, Receiver};
use alloc::format;

impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        // TODO: Add other fields here.
        let mut message = serializer.serialize_struct(self.performative.as_str(), 3)?;
        // message.serialize_field("sender", &self.sender)?;
        message.serialize_field("receiver", &self.receiver)?;
        match &self.content {
            Content::Structured(c) => {
                // TODO: Allow this to be configured
                message.serialize_field("lanuage", "fipa-sl0")?;
                message.serialize_field("content", &format!("\"{c}\""))?;
            }
            Content::Bytes(b) => {
                message.serialize_field("language", "bytes")?;
                // TODO: Encode as regular bytes when parsing support for acl messages is expanded.
                message
                    .serialize_field("content", &format!("\"{}\"", hex::encode(b.as_slice())))?;
            }
            Content::Other { kind, content } => {
                if let Some(kind) = kind {
                    message.serialize_field("language", kind)?;
                }
                message.serialize_field("content", &format!("\"{content}\""))?;
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
        use serde::ser::SerializeSeq;

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

impl serde::Serialize for OtherLanguage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            OtherLanguage::Ccl => "fipa-ccl",
            OtherLanguage::Kif => "fipa-kif",
            OtherLanguage::Rdf => "fipa-rdf",
        })
    }
}
