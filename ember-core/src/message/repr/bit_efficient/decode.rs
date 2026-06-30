use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use bstr::{BString, ByteSlice};

use crate::agent::aid::Aid;
use crate::message::content::fipa_sl::Sl0Content;
use crate::message::{Content, Message, Performative, Receiver};

pub(super) fn decode(bytes: &[u8]) -> Result<Message, ()> {
    let input = crate::util::parsing::BStr::from(bstr::BStr::new(bytes));
    parser::message(&input).map_err(|e| log::error!("bit-efficient decode error: {e}"))
}

enum MessageField {
    Receiver(Vec<Aid>),
    Language(BString),
    Ontology(String),
    Other(String, BString),
    Content(Vec<u8>),
}

fn build_message(
    perf: Performative,
    fields: Vec<Option<MessageField>>,
) -> Result<Message, &'static str> {
    let mut receiver: Option<Vec<Aid>> = None;
    let mut language: Option<BString> = None;
    let mut ontology: Option<String> = None;
    let mut other_fields = Vec::new();
    let mut content: Option<Vec<u8>> = None;

    for field in fields.into_iter().flatten() {
        match field {
            MessageField::Receiver(aids) => receiver = Some(aids),
            MessageField::Language(l) => language = Some(l),
            MessageField::Ontology(o) => ontology = Some(o),
            MessageField::Other(n, v) => other_fields.push((n, v)),
            MessageField::Content(c) => content = Some(c),
        }
    }

    let receiver = receiver.map(|mut r| match r.len() {
        1 => Receiver::Single(r.pop().expect("receiver should be of length 1")),
        _ => Receiver::Multiple(BTreeSet::from_iter(r)),
    });

    let content = if let Some(content) = content {
        Some(match language.as_ref().map(|l| l.as_bytes()) {
            Some(b"fipa-sl0") => {
                let parsed = Sl0Content::try_from_sl(content.as_bstr()).map_err(|e| {
                    log::error!("failed to parse SL0 content: {e}");
                    "sl0-content"
                })?;
                Content::FipaSl0(parsed)
            }
            Some(b"bytes") => Content::Bytes(content),
            None => Content::Other {
                kind: None,
                content: content.into(),
            },
            Some(l) => {
                log::warn!(
                    "unrecognised content language `{}`, treating as opaque",
                    bstr::BStr::new(l)
                );
                Content::Other {
                    kind: None,
                    content: content.into(),
                }
            }
        })
    } else {
        None
    };

    let other = (!other_fields.is_empty())
        .then(|| BTreeMap::from_iter(other_fields))
        .or(None);

    Ok(Message {
        performative: perf,
        receiver,
        ontology,
        other,
        content,
    })
}

peg::parser! {
    pub(super) grammar parser<'a>() for crate::util::parsing::BStr<'a> {
        use alloc::vec;
        use alloc::vec::Vec;
        use crate::message::repr::bit_efficient::codec::*;

        pub rule message() -> Message
            = message_id()
              [VERSION_1_0]
              p:performative()
              fields:message_parameter()*
              eoc()
            {?
                build_message(p, fields)
            }

        rule message_parameter() -> Option<MessageField>
            = [KW_SENDER] agent_identifier_skip() { None }
            / [KW_RECEIVER] aids:agent_identifier_collection() { Some(MessageField::Receiver(aids)) }
            / [KW_CONTENT] s:bin_string() { Some(MessageField::Content(s)) }
            / [KW_REPLY_WITH] bin_expression_skip() { None }
            / [KW_REPLY_BY] bin_datetime_skip() { None }
            / [KW_IN_REPLY_TO] bin_expression_skip() { None }
            / [KW_REPLY_TO] agent_identifier_collection() { None }
            / [KW_LANGUAGE] s:bin_expression_bytes() { Some(MessageField::Language(s.into())) }
            / [KW_ENCODING] bin_expression_skip() { None }
            / [KW_ONTOLOGY] s:bin_expression_bytes()
                {?
                    Ok(Some(
                        MessageField::Ontology(
                            String::from_utf8(s)
                                .map_err(|_| "utf8 ontology string")?
                        )
                    ))
                }
            / [KW_PROTOCOL] bin_word_skip() { None }
            / [KW_CONVERSATION_ID] bin_expression_skip() { None }
            / [0x00] n:bin_word() v:bin_expression_bytes()
                {?
                    let name = String::from_utf8(n).map_err(|_| "utf8 parameter name")?;
                    Ok(Some(MessageField::Other(name, v.into())))
                }


        rule bin_word() -> Vec<u8>
            = [BIN_WORD] s:$([b if b != 0x00]*) [0x00] { s.to_vec() }
            / [BIN_WORD_IDX] [_] { vec![] }

        rule bin_string() -> Vec<u8>
            = [BIN_STR_NULL] s:$([b if b != 0x00]*) [0x00] { s.to_vec() }
            / [BIN_STR_IDX] [_] { vec![] }
            / [BIN_STR_8] n:[_] s:$([_] *<{n as usize}>) { s.to_vec() }
            / [BIN_STR_16] hi:[_] lo:[_]
              s:$([_] *<{(hi as usize) << 8 | lo as usize}>) { s.to_vec() }
            / [BIN_STR_IDX2] [_] { vec![] }
            / [BIN_STR_32] b3:[_] b2:[_] b1:[_] b0:[_]
              s:$([_] *<{(b3 as usize) << 24 | (b2 as usize) << 16
                         | (b1 as usize) << 8 | b0 as usize}>) { s.to_vec() }

        rule bin_expression_bytes() -> Vec<u8>
            = w:bin_word()           { w }
            / s:bin_string()         { s }
            / [0xff] s:bin_string()  { s }
            / expr_start() bin_expr_skip()* expr_end() { vec![] }

        rule nul_term() = [b if b != 0x00]* [0x00]

        rule bin_word_skip()
            = [BIN_WORD] nul_term()
            / [BIN_WORD_IDX] [_]

        rule bin_string_skip()
            = [BIN_STR_NULL] nul_term()
            / [BIN_STR_IDX] [_]
            / [BIN_STR_8] n:[_] [_] *<{n as usize}>
            / [BIN_STR_16] hi:[_] lo:[_] [_] *<{(hi as usize) << 8 | lo as usize}>
            / [BIN_STR_IDX2] [_]
            / [BIN_STR_32] b3:[_] b2:[_] b1:[_] b0:[_]
              [_] *<{(b3 as usize) << 24 | (b2 as usize) << 16
                    | (b1 as usize) << 8 | b0 as usize}>

        rule coded_digits() = [b if b & 0xf0 == 0x00]* [b if b & 0xf0 != 0x00]

        rule bin_number_skip()
            = [0x12] coded_digits()
            / [0x13] coded_digits()

        rule expr_start()
            = [0x60]
            / [0x70] nul_term()
            / [0x71] [_]
            / [0x72] coded_digits()
            / [0x73] coded_digits()
            / [0x74] nul_term()
            / [0x75] [_]
            / [0x76] n:[_] [_] *<{n as usize}>
            / [0x77] hi:[_] lo:[_] [_] *<{(hi as usize) << 8 | lo as usize}>
            / [0x78] b3:[_] b2:[_] b1:[_] b0:[_]
              [_] *<{(b3 as usize) << 24 | (b2 as usize) << 16
                    | (b1 as usize) << 8 | b0 as usize}>
            / [0x79] [_]

        rule expr_end()
            = [0x40]
            / [0x50] nul_term()
            / [0x51] [_]
            / [0x52] coded_digits()
            / [0x53] coded_digits()
            / [0x54] nul_term()
            / [0x55] [_]
            / [0x56] n:[_] [_] *<{n as usize}>
            / [0x57] hi:[_] lo:[_] [_] *<{(hi as usize) << 8 | lo as usize}>
            / [0x58] b3:[_] b2:[_] b1:[_] b0:[_]
              [_] *<{(b3 as usize) << 24 | (b2 as usize) << 16
                    | (b1 as usize) << 8 | b0 as usize}>
            / [0x59] [_]

        rule bin_expr_skip()
            = bin_word_skip()
            / bin_string_skip()
            / bin_number_skip()
            / expr_start() bin_expr_skip()* expr_end()

        rule bin_expression_skip()
            = bin_expr_skip()
            / [0xff] bin_string_skip()

        rule bin_date_skip() = [_] *<{9}>

        rule bin_datetime_skip()
            = [0x20] bin_date_skip()
            / [0x21] bin_date_skip()
            / [0x22] bin_date_skip()
            / [0x24] bin_date_skip() [_]
            / [0x25] bin_date_skip() [_]
            / [0x26] bin_date_skip() [_]

        rule url_collection_skip() = bin_word_skip()* eoc()

        rule agent_identifier_collection() -> Vec<Aid>
            = aids:agent_identifier()* eoc() { aids }

        rule agent_identifier_collection_skip()
            = agent_identifier_skip()* eoc()

        rule agent_identifier() -> Aid
            = [0x02]
              name:bin_word()
              ([AID_TAG_ADDRESSES] url_collection_skip())?
              ([AID_TAG_RESOLVERS] agent_identifier_collection())?
              ([AID_TAG_USER_DEF] bin_word_skip() bin_expression_skip())*
              eoc()
            {?
                let s = core::str::from_utf8(&name).map_err(|_| "AID name not UTF-8")?;
                s.parse::<Aid>().map_err(|_| "bad AID")
            }

        rule agent_identifier_skip()
            = [0x02]
              bin_word_skip()
              ([AID_TAG_ADDRESSES] url_collection_skip())?
              ([AID_TAG_RESOLVERS] agent_identifier_collection_skip())?
              ([AID_TAG_USER_DEF] bin_word_skip() bin_expression_skip())*
              eoc()

        rule performative() -> Performative
            = [PERF_ACCEPT_PROPOSAL] { Performative::AcceptProposal }
            / [PERF_AGREE]           { Performative::Agree }
            / [PERF_CANCEL]          { Performative::Cancel }
            / [PERF_CFP]             { Performative::Cfp }
            / [PERF_CONFIRM]         { Performative::Confirm }
            / [PERF_DISCONFIRM]      { Performative::Disconfirm }
            / [PERF_FAILURE]         { Performative::Failure }
            / [PERF_INFORM]          { Performative::Inform }
            / [PERF_INFORM_IF]       { Performative::InformIf }
            / [PERF_INFORM_REF]      { Performative::InformRef }
            / [PERF_NOT_UNDERSTOOD]  { Performative::NotUnderstood }
            / [PERF_PROPAGATE]       { Performative::Propagate }
            / [PERF_PROPOSE]         { Performative::Propose }
            / [PERF_PROXY]           { Performative::Proxy }
            / [PERF_QUERY_IF]        { Performative::QueryIf }
            / [PERF_QUERY_REF]       { Performative::QueryRef }
            / [PERF_REFUSE]          { Performative::Refuse }
            / [PERF_REJECT_PROPOSAL] { Performative::RejectProposal }
            / [PERF_REQUEST]         { Performative::Request }
            / [PERF_REQUEST_WHEN]    { Performative::RequestWhen }
            / [PERF_REQUEST_WHENEVER]{ Performative::RequestWhenever }
            / [PERF_SUBSCRIBE]       { Performative::Subscribe }

        rule eoc() = [END_OF_COLLECTION]

        rule message_id()
            = [MESSAGE_ID]
            / [_] {? Err("unsupported message id") }

    }
}
