use alloc::collections::BTreeSet;
use alloc::string::ToString;
use alloc::vec::Vec;

use bstr::ByteSlice;

use crate::agent::aid::Aid;
use crate::message::content::fipa_sl::Sl0Content;
use crate::message::{Content, Message, Performative, Receiver};

pub(super) fn decode(bytes: &[u8]) -> Result<Message, ()> {
    let input = crate::util::parsing::BStr::from(bstr::BStr::new(bytes));
    parser::message(&input).map_err(|e| log::error!("bit-efficient decode error: {e}"))
}

enum MessageField {
    Sender(Aid),
    Receiver(Vec<Aid>),
    Language(Vec<u8>),
    Content(Vec<u8>),
    Ontology(Vec<u8>),
}

fn build_message(
    perf: Performative,
    fields: Vec<Option<MessageField>>,
) -> Result<Message, &'static str> {
    let mut sender: Option<Aid> = None;
    let mut receiver: Option<Vec<Aid>> = None;
    let mut language: Option<Vec<u8>> = None;
    let mut content_bytes: Option<Vec<u8>> = None;
    let mut ontology_bytes: Option<Vec<u8>> = None;

    for field in fields.into_iter().flatten() {
        match field {
            MessageField::Sender(a) => sender = Some(a),
            MessageField::Receiver(aids) => receiver = Some(aids),
            MessageField::Language(l) => language = Some(l),
            MessageField::Content(c) => content_bytes = Some(c),
            MessageField::Ontology(o) => ontology_bytes = Some(o),
        }
    }

    let aids = receiver.ok_or("missing :receiver")?;
    let receiver = match aids.len() {
        1 => Receiver::Single(aids.into_iter().next().unwrap()),
        _ => Receiver::Multiple(BTreeSet::from_iter(aids)),
    };

    let content_bytes = content_bytes.ok_or("missing :content")?;

    let content = match language.as_deref() {
        Some(b"fipa-sl0") => {
            let parsed = Sl0Content::try_from_sl(content_bytes.as_bstr()).map_err(|e| {
                log::error!("failed to parse SL0 content: {e}");
                "sl0-content"
            })?;
            Content::FipaSl0(parsed)
        }
        Some(b"bytes") => Content::Bytes(content_bytes),
        None => Content::Other {
            kind: None,
            content: content_bytes.into(),
        },
        Some(l) => {
            log::warn!(
                "unrecognised content language `{}`, treating as opaque",
                bstr::BStr::new(l)
            );
            Content::Other {
                kind: None,
                content: content_bytes.into(),
            }
        }
    };

    let ontology = ontology_bytes
        .map(|b| core::str::from_utf8(&b).unwrap_or_default().to_string())
        .filter(|s| !s.is_empty());

    Ok(Message {
        performative: perf,
        sender,
        receiver,
        reply_to: None,
        ontology,
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
            = [KW_SENDER]          a:agent_identifier()               { Some(MessageField::Sender(a)) }
            / [KW_RECEIVER]        aids:agent_identifier_collection() { Some(MessageField::Receiver(aids)) }
            / [KW_CONTENT]         s:bin_string()                     { Some(MessageField::Content(s)) }
            / [KW_REPLY_WITH]      bin_expression_skip()              { None }
            / [KW_REPLY_BY]        bin_datetime_skip()                { None }
            / [KW_IN_REPLY_TO]     bin_expression_skip()              { None }
            / [KW_REPLY_TO]        agent_identifier_collection()      { None }
            / [KW_LANGUAGE]        s:bin_expression_bytes()           { Some(MessageField::Language(s)) }
            / [KW_ENCODING]        bin_expression_skip()              { None }
            / [KW_ONTOLOGY]        s:bin_expression_bytes()           { Some(MessageField::Ontology(s)) }
            / [KW_PROTOCOL]        bin_word_skip()                    { None }
            / [KW_CONVERSATION_ID] bin_expression_skip()              { None }
            / [0x00]               bin_word_skip() bin_expression_skip() { None }


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
