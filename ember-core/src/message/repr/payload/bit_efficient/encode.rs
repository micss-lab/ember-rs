use alloc::string::ToString;
use alloc::vec::Vec;

use crate::agent::aid::Aid;
use crate::message::{Content, Performative, Receiver};

use super::codec::*;

pub(super) fn push_aid(aid: &Aid, out: &mut Vec<u8>) {
    out.push(0x02);
    push_bin_word(aid.to_string().as_bytes(), out);
    out.push(END_OF_COLLECTION);
}

pub(super) fn push_recipient_expr(receiver: &Receiver, out: &mut Vec<u8>) {
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

pub(super) fn push_content_and_language(content: &Content, out: &mut Vec<u8>) {
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
        Content::Bdil(c) => {
            out.push(KW_LANGUAGE);
            push_bin_word(b"ember-bdil", out);
            out.push(KW_ENCODING);
            push_bin_word(b"bit-efficient", out);
            out.push(KW_CONTENT);
            push_bin_string(
                &ember_bdi_bdil::binary::encode(c).expect("failed to encode bdil payload"),
                out,
            );
        }
        Content::Other { language, content } => {
            if let Some(kind) = language {
                out.push(KW_LANGUAGE);
                push_bin_word(kind.as_bytes(), out);
            }
            out.push(KW_CONTENT);
            push_bin_string(content.as_slice(), out);
        }
    }
}

pub(super) fn push_bin_word(word: &[u8], out: &mut Vec<u8>) {
    out.push(BIN_WORD);
    out.extend_from_slice(word);
    out.push(0x00);
}

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

pub(super) fn performative_code(p: Performative) -> u8 {
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
