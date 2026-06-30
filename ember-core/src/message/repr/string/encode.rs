use alloc::string::String;
use alloc::vec::Vec;

use core::fmt::{self, Write};

use crate::agent::aid::Aid;
use crate::message::{Content, Message, OtherLanguage, Receiver};

pub(super) fn encode(message: &Message, out: &mut Vec<u8>) -> fmt::Result {
    let mut result = String::new();
    write!(result, "({}", message.performative.as_str())?;
    if let Some(sender) = &message.sender {
        result.push_str(" :sender ");
        encode_aid(sender, &mut result)?;
    }
    result.push_str(" :receiver ");
    encode_receiver(&message.receiver, &mut result)?;
    encode_content(&message.content, &mut result)?;
    if let Some(ontology) = &message.ontology {
        write!(result, " :ontology {ontology}")?;
    }
    result.push(')');
    out.extend(result.into_bytes());
    Ok(())
}

fn encode_aid(aid: &Aid, out: &mut String) -> fmt::Result {
    write!(out, "(agent-identifier :name {aid})")
}

fn encode_receiver(receiver: &Receiver, out: &mut String) -> fmt::Result {
    match receiver {
        Receiver::Single(aid) => encode_aid(aid, out),
        Receiver::Multiple(aids) => {
            out.push_str("(sequence");
            for aid in aids {
                out.push(' ');
                encode_aid(aid, out)?;
            }
            out.push(')');
            Ok(())
        }
    }
}

fn encode_content(content: &Content, out: &mut String) -> fmt::Result {
    match content {
        Content::FipaSl0(c) => {
            write!(out, " :language fipa-sl0 :content \"{c}\"")
        }
        Content::Bytes(b) => {
            use base64ct::{Base64, Encoding};
            write!(
                out,
                " :language bytes :content \"{}\"",
                Base64::encode_string(b)
            )
        }
        Content::Other { kind, content } => {
            if let Some(kind) = kind {
                write!(out, " :language {}", language_name(kind))?;
            }
            write!(out, " :content \"{content}\"")
        }
    }
}

fn language_name(lang: &OtherLanguage) -> &'static str {
    match lang {
        OtherLanguage::Ccl => "fipa-ccl",
        OtherLanguage::Kif => "fipa-kif",
        OtherLanguage::Rdf => "fipa-rdf",
    }
}
