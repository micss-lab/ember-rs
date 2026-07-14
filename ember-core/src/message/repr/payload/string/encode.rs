use alloc::string::String;
use alloc::vec::Vec;

use core::fmt::{self, Write};

use crate::agent::aid::Aid;
use crate::message::{Content, Message, Receiver};

pub(super) fn encode(message: &Message, out: &mut Vec<u8>) -> fmt::Result {
    let mut result = String::new();
    write!(result, "({}", message.performative.as_str())?;
    if let Some(ref receiver) = message.receiver {
        result.push_str(" :receiver ");
        encode_receiver(receiver, &mut result)?;
    }
    if let Some(ref ontology) = message.ontology {
        write!(result, " :ontology {ontology}")?;
    }
    if let Some(ref content) = message.content {
        encode_content_and_language(content, &mut result)?;
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

fn encode_content_and_language(content: &Content, out: &mut String) -> fmt::Result {
    let language = content.language();
    match content {
        Content::FipaSl0(c) => {
            write!(out, " :language {language} :content \"{c}\"")
        }
        Content::Bytes(b) => {
            use base64ct::{Base64, Encoding};
            write!(
                out,
                " :language {language} :X-content-encoding base64 :content \"{}\"",
                Base64::encode_string(b)
            )
        }
        Content::Bdil(c) => {
            use base64ct::{Base64, Encoding};
            write!(
                out,
                " :language {language} :X-content-encoding base64 :encoding bit-efficient :content \"{}\"",
                Base64::encode_string(&ember_bdi_bdil::binary::encode(c))
            )
        }
        Content::Other { language, content } => {
            use bstr::ByteSlice;
            if let Some(kind) = language {
                write!(out, " :language {kind}")?;
            }
            if content.is_utf8() {
                write!(out, " :content \"{content}\"")
            } else {
                use base64ct::{Base64, Encoding};
                write!(
                    out,
                    " :X-content-encoding bytes :content \"{}\"",
                    Base64::encode_string(content)
                )
            }
        }
    }
}
