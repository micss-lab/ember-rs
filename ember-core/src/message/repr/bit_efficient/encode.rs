use alloc::string::ToString;
use alloc::vec::Vec;

use crate::agent::aid::Aid;
use crate::message::{Content, Message, OtherLanguage, Performative, Receiver};

use super::codec::*;

pub(super) fn encode(message: &Message, out: &mut Vec<u8>) {
    out.push(MESSAGE_ID);
    out.push(VERSION_1_0);
    out.push(performative_code(message.performative));

    if let Some(sender) = &message.sender {
        out.push(KW_SENDER);
        push_aid(sender, out);
    }

    out.push(KW_RECEIVER);
    push_recipient_expr(&message.receiver, out);

    push_content(&message.content, out);

    if let Some(ontology) = &message.ontology {
        out.push(KW_ONTOLOGY);
        push_bin_word(ontology.as_bytes(), out);
    }

    out.push(END_OF_COLLECTION);
}

// AgentIdentifier = 0x02 AgentName EndOfCollection   (no addresses/resolvers)
fn push_aid(aid: &Aid, out: &mut Vec<u8>) {
    out.push(0x02);
    push_bin_word(aid.to_string().as_bytes(), out);
    out.push(END_OF_COLLECTION);
}

// RecipientExpr = AgentIdentifierCollection = AgentIdentifier* EndOfCollection
fn push_recipient_expr(receiver: &Receiver, out: &mut Vec<u8>) {
    match receiver {
        Receiver::Single(aid) => {
            push_aid(aid, out);
        }
        Receiver::Multiple(aids) => {
            for aid in aids {
                push_aid(aid, out);
            }
        }
    }
    out.push(END_OF_COLLECTION);
}

fn push_content(content: &Content, out: &mut Vec<u8>) {
    match content {
        Content::FipaSl0(c) => {
            out.push(KW_LANGUAGE);
            push_bin_word(b"fipa-sl0", out);
            out.push(KW_CONTENT);
            push_bin_string(c.to_string().as_bytes(), out);
        }
        Content::Bytes(b) => {
            out.push(KW_LANGUAGE);
            push_bin_word(b"bytes", out);
            out.push(KW_CONTENT);
            push_bin_string(b, out);
        }
        Content::Other { kind, content } => {
            if let Some(kind) = kind {
                out.push(KW_LANGUAGE);
                push_bin_word(language_name(kind).as_bytes(), out);
            }
            out.push(KW_CONTENT);
            push_bin_string(content.as_slice(), out);
        }
    }
}

// BinWord = 0x10 Word 0x00
fn push_bin_word(word: &[u8], out: &mut Vec<u8>) {
    out.push(BIN_WORD);
    out.extend_from_slice(word);
    out.push(0x00);
}

// BinString: 0x17 Len16 ByteSeq  or  0x19 Len32 ByteSeq
fn push_bin_string(bytes: &[u8], out: &mut Vec<u8>) {
    if bytes.len() <= u16::MAX as usize {
        out.push(BIN_STR_16);
        out.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
    } else {
        out.push(BIN_STR_32);
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    }
    out.extend_from_slice(bytes);
}

fn language_name(lang: &OtherLanguage) -> &'static str {
    match lang {
        OtherLanguage::Ccl => "fipa-ccl",
        OtherLanguage::Kif => "fipa-kif",
        OtherLanguage::Rdf => "fipa-rdf",
    }
}

fn performative_code(p: Performative) -> u8 {
    match p {
        Performative::AcceptProposal => PERF_ACCEPT_PROPOSAL,
        Performative::Agree => PERF_AGREE,
        Performative::Cancel => PERF_CANCEL,
        Performative::Cfp => PERF_CFP,
        Performative::Confirm => PERF_CONFIRM,
        Performative::Disconfirm => PERF_DISCONFIRM,
        Performative::Failure => PERF_FAILURE,
        Performative::Inform => PERF_INFORM,
        Performative::InformIf => PERF_INFORM_IF,
        Performative::InformRef => PERF_INFORM_REF,
        Performative::NotUnderstood => PERF_NOT_UNDERSTOOD,
        Performative::Propose => PERF_PROPOSE,
        Performative::QueryIf => PERF_QUERY_IF,
        Performative::QueryRef => PERF_QUERY_REF,
        Performative::Refuse => PERF_REFUSE,
        Performative::RejectProposal => PERF_REJECT_PROPOSAL,
        Performative::Request => PERF_REQUEST,
        Performative::RequestWhen => PERF_REQUEST_WHEN,
        Performative::RequestWhenever => PERF_REQUEST_WHENEVER,
        Performative::Subscribe => PERF_SUBSCRIBE,
        Performative::Proxy => PERF_PROXY,
        Performative::Propagate => PERF_PROPAGATE,
    }
}
