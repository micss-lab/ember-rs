use alloc::string::ToString;
use alloc::vec::Vec;

use chrono::{DateTime, Datelike, FixedOffset, Timelike};

use crate::agent::aid::Aid;
use crate::message::AclRepresentation;

use super::codec::*;

pub(super) fn push_null_terminated_string(bytes: &[u8], out: &mut Vec<u8>) {
    out.extend_from_slice(bytes);
    out.push(0x00);
}

/// AID encoding for the envelope: `0x02 AgentName EndOfCollection`, where `AgentName` is a bare
/// `NullTerminatedString` (unlike the ACL payload's AID, whose name is a tagged `BinWord`).
pub(super) fn push_aid(aid: &Aid, out: &mut Vec<u8>) {
    out.push(AID_TAG);
    push_null_terminated_string(aid.to_string().as_bytes(), out);
    out.push(END_OF_COLLECTION);
}

pub(super) fn push_agent_identifier_sequence(aids: &[Aid], out: &mut Vec<u8>) {
    for aid in aids {
        push_aid(aid, out);
    }
    out.push(END_OF_COLLECTION);
}

pub(super) fn push_acl_representation(repr: &AclRepresentation, out: &mut Vec<u8>) {
    match repr {
        AclRepresentation::BitEfficient => out.push(ACL_REP_BIT_EFFICIENT),
        AclRepresentation::String => out.push(ACL_REP_STRING),
        AclRepresentation::Other(name) => {
            out.push(ACL_REP_USER_DEFINED);
            push_null_terminated_string(name.as_bytes(), out);
        }
    }
}

/// `BinDateTimeToken`: always encoded as an absolute (`0x20`), untyped date. `date` is converted
/// to UTC first, since the wire format has no timezone-offset field of its own.
pub(super) fn push_date(date: &DateTime<FixedOffset>, out: &mut Vec<u8>) {
    let utc = date.with_timezone(&chrono::Utc);

    out.push(DATE_ABS);
    out.extend_from_slice(&(utc.year() as u16).to_be_bytes());
    out.push(utc.month() as u8);
    out.push(utc.day() as u8);
    out.push(utc.hour() as u8);
    out.push(utc.minute() as u8);
    out.push(utc.second() as u8);
    out.extend_from_slice(&(utc.timestamp_subsec_millis() as u16).to_be_bytes());
}
