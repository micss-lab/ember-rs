use alloc::string::String;
use core::fmt::Write;

use crate::agent::aid::Aid;
use crate::message::{Content, Message, OtherLanguage, Receiver};

pub(super) fn encode(message: &Message) -> alloc::vec::Vec<u8> {
    let mut out = String::new();
    write!(out, "({}", message.performative.as_str()).unwrap();
    if let Some(sender) = &message.sender {
        out.push_str(" :sender ");
        encode_aid(sender, &mut out);
    }
    out.push_str(" :receiver ");
    encode_receiver(&message.receiver, &mut out);
    encode_content(&message.content, &mut out);
    if let Some(ontology) = &message.ontology {
        write!(out, " :ontology {ontology}").unwrap();
    }
    out.push(')');
    out.into_bytes()
}

fn encode_aid(aid: &Aid, out: &mut String) {
    write!(out, "(agent-identifier :name {aid})").unwrap();
}

fn encode_receiver(receiver: &Receiver, out: &mut String) {
    match receiver {
        Receiver::Single(aid) => encode_aid(aid, out),
        Receiver::Multiple(aids) => {
            out.push_str("(sequence");
            for aid in aids {
                out.push(' ');
                encode_aid(aid, out);
            }
            out.push(')');
        }
    }
}

fn encode_content(content: &Content, out: &mut String) {
    match content {
        Content::Structured(c) => {
            write!(out, " :language fipa-sl0 :content \"{c}\"").unwrap();
        }
        Content::Bytes(b) => {
            use base64ct::{Base64, Encoding};
            write!(out, " :language bytes :content \"{}\"", Base64::encode_string(b)).unwrap();
        }
        Content::Other { kind, content } => {
            if let Some(kind) = kind {
                write!(out, " :language {}", language_name(kind)).unwrap();
            }
            write!(out, " :content \"{content}\"").unwrap();
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
