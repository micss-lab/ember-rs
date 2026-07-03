use alloc::string::String;

use crate::agent::aid::Aid;
use crate::message::{Message, Performative, Receiver};

use super::builder::{AidBuilder, AidField, MessageBuilder, MessageField};

peg::parser! {
    pub(super) grammar messsage<'a>() for crate::util::parsing::BStr<'a> {
        use alloc::vec;

        use bstr::ByteSlice;

        rule _ = [c if c.is_ascii_whitespace()]*

        pub rule message() -> Message
            = lbrace() _ p:performative() _ fs:(message_field() ** _) _ rbrace()
        {?
            let mut builder = MessageBuilder::default();
            for field in fs.into_iter().flatten() {
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

        rule performative() -> Performative
            = w:word() {? w.to_str_lossy().parse().or(Err("performative")) }

        rule message_field() -> Option<MessageField>
            = ":receiver" _ r:receiver() { Some(MessageField::Receiver(r)) }
            / ":language" _ l:word() { Some(MessageField::Language(l)) }
            / ":ontology" _ o:word()
                {?
                    let ontology = String::from_utf8(o.into())
                        .map_err(|_| "utf8 ontology string")?;
                        Ok(Some(MessageField::Ontology(ontology)))
                }
            / ":content" _ c:string() { Some(MessageField::Content(c)) }
            / ":" n:word() _ v:string()
                {?
                    // TODO: Implement this.
                    Err("unimplemented fields in string acl repr decode")
                }

        rule receiver() -> Receiver
            = aid:agent_identifier() { Receiver::Single(aid)}

        rule agent_identifier() -> Aid
            = lbrace() _ "agent-identifier" _ fs:(aid_field() ** _) _ rbrace()
        {?
            let mut builder = AidBuilder::default();
            for field in fs {
                match field {
                    AidField::Name(n) => builder.set_name(n)?,
                }
            }
            builder.build()
        }

        rule aid_field() -> AidField
            = ":name" _ n:word() { AidField::Name(n) }

        /// Regular quoted string literal.
        rule string_literal() -> bstr::BString
            = quote() s:$(("\\\"" / [^b'"'])*) quote() { s.into() }

        /// Single word that is a valid variable name.
        rule word() -> bstr::BString
            = s:$([^(0x00..=0x20 | b'(' | b')' | b'#' | b'0'..=b'9' | b'-' | b'@')][^(0x00..=0x20 | b'(' | b')')]*) {
            s.into()
        }

        rule string() -> bstr::BString
            = string_literal()

        // ====================
        //       Symbols
        // ====================

        rule dot() = _ [b'.'] _

        /// Whether the rule matched a sign.
        ///
        /// Negative is represented as true.
        rule sign() -> Option<bool> = _ s:[b'-' | b'+']? _ { s.map(|s| s == b'-') }

        /// Whether the rule matched a sign that is negative.
        rule neg_sign() -> bool = _ s:sign() _ { s.is_some_and(|s| s) }

        rule lbrace() = _ [b'('] _
        rule rbrace() = _ [b')'] _

        rule semi() = _ [b';'] _
        rule colon() = _ [b':'] _

        rule vert() = _ [b'|'] _

        rule eq() = _ [b'='] _

        rule time_sep() = _ [b'T'] _

        rule hashtag() = _ [b'#'] _

        rule quote() = _ [b'"'] _
    }
}
