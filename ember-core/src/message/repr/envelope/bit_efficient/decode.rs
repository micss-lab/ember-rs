use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use bstr::BString;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};

use crate::agent::aid::Aid;
use crate::message::repr::envelope::bit_efficient::codec::*;
use crate::message::{AclRepresentation, MessageEnvelope};

enum Field {
    To(Vec<Aid>),
    From(Aid),
    Other(String, BString),
}

#[derive(Default)]
struct EnvelopeBuilder {
    to: Vec<Aid>,
    from: Option<Aid>,
    other: BTreeMap<String, BString>,
}

impl EnvelopeBuilder {
    fn set_to(&mut self, aids: Vec<Aid>) {
        self.to = aids;
    }

    fn set_from(&mut self, aid: Aid) -> Result<(), &'static str> {
        if self.from.replace(aid).is_some() {
            log::error!("duplicate field `from`");
            return Err("from");
        }
        Ok(())
    }

    fn add_other(&mut self, key: String, value: BString) {
        self.other.insert(key, value);
    }

    fn build(
        self,
        acl_representation: AclRepresentation,
        date: DateTime<FixedOffset>,
    ) -> Result<MessageEnvelope, &'static str> {
        let Some(from) = self.from else {
            log::error!("envelope is missing required field `from`");
            return Err("from");
        };
        Ok(MessageEnvelope {
            to: self.to,
            from,
            date,
            acl_representation,
            other: (!self.other.is_empty()).then_some(self.other),
        })
    }
}

peg::parser! {
    pub(super) grammar parser<'a>() for crate::util::parsing::BStr<'a> {

        pub rule envelope() -> MessageEnvelope
            = [BASE_MSG_ID]
              env_len()
              acl_representation:acl_representation()
              date:bin_date_time()
              fields:parameter()*
              eoc()
            {?
                let mut builder = EnvelopeBuilder::default();
                for field in fields.into_iter().flatten() {
                    match field {
                        Field::To(aids) => builder.set_to(aids),
                        Field::From(aid) => builder.set_from(aid)?,
                        Field::Other(k, v) => builder.add_other(k, v),
                    }
                }
                builder.build(acl_representation, date)
            }

        // The length itself was already validated and consumed by `super::read_envelope_len`
        // when slicing out exactly this envelope's bytes; here we only need to consume the same
        // number of header bytes so the rest of the grammar lines up.
        rule env_len()
            = [0x00] [0x00] [_] [_] [_] [_]
            / [_] [_]

        rule parameter() -> Option<Field>
            = [PARAM_TO] aids:agent_identifier_sequence() { Some(Field::To(aids)) }
            / [PARAM_FROM] aid:agent_identifier() { Some(Field::From(aid)) }
            / [PARAM_ACL_REPRESENTATION] acl_representation_skip() { None }
            / [PARAM_COMMENTS] null_terminated_string_skip() { None }
            / [PARAM_PAYLOAD_LENGTH] bin_number_skip() { None }
            / [PARAM_PAYLOAD_ENCODING] null_terminated_string_skip() { None }
            / [PARAM_INTENDED_RECEIVER] agent_identifier_sequence_skip() { None }
            / [PARAM_RECEIVED] received_object_skip() { None }
            / [PARAM_TRANSPORT_BEHAVIOUR] any_skip() { None }
            / [PARAM_USER_DEFINED] k:null_terminated_string() v:null_terminated_string()
                {?
                    let key = String::from_utf8(k).map_err(|_| "user-defined parameter key not utf-8")?;
                    Ok(Some(Field::Other(key, v.into())))
                }

        rule acl_representation() -> AclRepresentation
            = [ACL_REP_USER_DEFINED] s:null_terminated_string()
                {?
                    Ok(AclRepresentation::Other(
                        String::from_utf8(s).map_err(|_| "acl-representation not utf-8")?
                    ))
                }
            / [ACL_REP_BIT_EFFICIENT] { AclRepresentation::BitEfficient }
            / [ACL_REP_STRING] { AclRepresentation::String }
            / [ACL_REP_XML] { AclRepresentation::Other("fipa.acl.rep.xml.std".into()) }

        rule acl_representation_skip()
            = [ACL_REP_USER_DEFINED] null_terminated_string_skip()
            / [ACL_REP_BIT_EFFICIENT]
            / [ACL_REP_STRING]
            / [ACL_REP_XML]

        rule agent_identifier() -> Aid
            = [AID_TAG]
              name:null_terminated_string()
              ([AID_ADDRESSES] url_sequence_skip())?
              ([AID_RESOLVERS] agent_identifier_sequence_skip())?
              ([AID_USER_DEFINED] null_terminated_string_skip() any_skip())*
              eoc()
            {?
                let s = core::str::from_utf8(&name).map_err(|_| "AID name not UTF-8")?;
                s.parse::<Aid>().map_err(|_| "bad AID")
            }

        rule agent_identifier_skip()
            = [AID_TAG]
              null_terminated_string_skip()
              ([AID_ADDRESSES] url_sequence_skip())?
              ([AID_RESOLVERS] agent_identifier_sequence_skip())?
              ([AID_USER_DEFINED] null_terminated_string_skip() any_skip())*
              eoc()

        rule agent_identifier_sequence() -> Vec<Aid>
            = aids:agent_identifier()* eoc() { aids }

        rule agent_identifier_sequence_skip()
            = agent_identifier_skip()* eoc()

        rule url_sequence_skip()
            = null_terminated_string_skip()* eoc()

        rule received_object_skip()
            = null_terminated_string_skip()                             // By = URL
              bin_date_time_skip()                                      // Date
              ([RECEIVED_FROM] null_terminated_string_skip())?          // From = 0x02 URL
              ([RECEIVED_ID] null_terminated_string_skip())?            // Id   = 0x03 NullTerminatedString
              ([RECEIVED_VIA] null_terminated_string_skip())?           // Via  = 0x04 NullTerminatedString
              ([PARAM_USER_DEFINED] null_terminated_string_skip() null_terminated_string_skip())*
              eoc()

        rule null_terminated_string() -> Vec<u8>
            = s:$([b if b != 0x00]*) [0x00] { s.to_vec() }

        rule null_terminated_string_skip()
            = [b if b != 0x00]* [0x00]

        rule any_skip()
            = [ANY_STRING] null_terminated_string_skip()
            / [ANY_BYTES_8] n:[_] [_] *<{n as usize}>
            / [ANY_BYTES_16] hi:[_] lo:[_] [_] *<{(hi as usize) << 8 | lo as usize}>
            / [ANY_BYTES_32] b3:[_] b2:[_] b1:[_] b0:[_]
              [_] *<{(b3 as usize) << 24 | (b2 as usize) << 16
                     | (b1 as usize) << 8 | b0 as usize}>

        rule coded_digits() = [b if b & 0xf0 == 0x00]* [b if b & 0xf0 != 0x00]

        rule bin_number_skip()
            = [NUM_DECIMAL] coded_digits()
            / [NUM_HEX] coded_digits()

        rule bin_date_time() -> DateTime<FixedOffset>
            = [DATE_ABS] d:bin_date() { d }
            / [DATE_ABS_TD] d:bin_date() [_] { d }
            / [DATE_REL_POS] bin_date_skip() {? Err("relative dates are not supported") }
            / [DATE_REL_NEG] bin_date_skip() {? Err("relative dates are not supported") }
            / [DATE_REL_POS_TD] bin_date_skip() [_] {? Err("relative dates are not supported") }
            / [DATE_REL_NEG_TD] bin_date_skip() [_] {? Err("relative dates are not supported") }

        rule bin_date_time_skip()
            = [DATE_ABS] bin_date_skip()
            / [DATE_ABS_TD] bin_date_skip() [_]
            / [DATE_REL_POS] bin_date_skip()
            / [DATE_REL_NEG] bin_date_skip()
            / [DATE_REL_POS_TD] bin_date_skip() [_]
            / [DATE_REL_NEG_TD] bin_date_skip() [_]

        rule bin_date() -> DateTime<FixedOffset>
            = y_hi:[_] y_lo:[_] month:[_] day:[_] hour:[_] minute:[_] second:[_] ms_hi:[_] ms_lo:[_]
            {?
                let year = ((y_hi as i32) << 8) | y_lo as i32;
                let ms = ((ms_hi as u32) << 8) | ms_lo as u32;
                let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32)
                    .ok_or("invalid date")?;
                let time = NaiveTime::from_hms_milli_opt(hour as u32, minute as u32, second as u32, ms)
                    .ok_or("invalid time")?;
                Ok(NaiveDateTime::new(date, time).and_utc().into())
            }

        rule bin_date_skip() = [_] *<{9}>

        rule eoc() = [END_OF_COLLECTION]

    }
}
