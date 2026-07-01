use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::agent::aid::Aid;
use crate::message::repr::payload::bit_efficient::codec::*;
use crate::message::{Message, Performative, Receiver};

use super::builder::{MessageBuilder, MessageField};

peg::parser! {
    pub(super) grammar parser<'a>() for crate::util::parsing::BStr<'a> {

        pub rule message() -> Message
            = message_id()
              [VERSION_1_0]
              p:performative()
              fields:message_parameter()*
              eoc()
            {?
                let mut builder = MessageBuilder::default();
                for field in fields.into_iter().flatten() {
                    match field {
                        MessageField::Receiver(r) => builder.set_receiver(r)?,
                        MessageField::Language(l) => builder.set_language(l)?,
                        MessageField::Ontology(o) => builder.set_ontology(o)?,
                        MessageField::Content(c) => builder.set_content(c)?,
                        MessageField::Other(n, v) => builder.add_other_field(n, v),
                    }
                }
                builder.build(p)
            }

        rule message_parameter() -> Option<MessageField>
            = [KW_SENDER] agent_identifier_skip() { None }
            / [KW_RECEIVER] aids:agent_identifier_collection()
                {
                    let mut aids = aids;
                    let receiver = match aids.len() {
                        1 => Receiver::Single(aids.pop().expect("aids should be length 1")),
                        _ => Receiver::Multiple(BTreeSet::from_iter(aids)),
                    };
                    Some(MessageField::Receiver(receiver))
                }
            / [KW_CONTENT] s:bin_string() { Some(MessageField::Content(s.into())) }
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
