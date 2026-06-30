use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use bstr::{BString, ByteSlice};

use crate::agent::aid::Aid;
use crate::message::content::fipa_sl::Sl0Content;
use crate::message::{Content, Message, Performative, Receiver};

type Result<T> = core::result::Result<T, &'static str>;

enum MessageField {
    Receiver(Receiver),
    Language(BString),
    Content(BString),
    Ontology(String),
    // TODO: Use these.
    #[allow(unused)]
    Other(String, BString),
}

#[derive(Default)]
struct MessageBuilder {
    receiver: Option<Receiver>,
    language: Option<BString>,
    ontology: Option<String>,
    other: Vec<(String, BString)>,
    content: Option<BString>,
}

impl MessageBuilder {
    fn build(self, performative: Performative) -> Result<Message> {
        let content = if let Some(content) = self.content {
            Some(match self.language.as_ref().map(|l| l.as_slice()) {
                Some(b"fipa-sl0") => {
                    Content::FipaSl0(Sl0Content::try_from_sl(content.as_bstr()).map_err(|e| {
                        log::error!("failed to parse content as sl0: {e}");
                        "content"
                    })?)
                }
                Some(b"bytes") => {
                    use base64ct::{Base64, Encoding};
                    Content::Bytes(Base64::decode_vec(content.to_str_lossy().as_ref()).map_err(
                        |_| {
                            log::error!("failed to parse bytes content from base64");
                            "bytes-content"
                        },
                    )?)
                }
                None => Content::Other {
                    kind: None,
                    content: content.into(),
                },
                Some(l) => {
                    log::warn!(
                        "unrecognised content language `{}`, treating as opaque string",
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

        let other = (!self.other.is_empty())
            .then(|| BTreeMap::from_iter(self.other))
            .or(None);

        Ok(Message {
            performative,
            receiver: self.receiver,
            ontology: self.ontology,
            other,
            content,
        })
    }

    fn set_receiver(&mut self, receiver: Receiver) -> Result<()> {
        if self.receiver.replace(receiver).is_some() {
            return Err("set_receiver");
        }
        Ok(())
    }

    fn set_language(&mut self, language: BString) -> Result<()> {
        if self.language.replace(language).is_some() {
            return Err("set_language");
        }
        Ok(())
    }

    fn set_content(&mut self, content: BString) -> Result<()> {
        if self.content.replace(content).is_some() {
            return Err("set_content");
        }
        Ok(())
    }

    fn set_ontology(&mut self, ontology: String) -> Result<()> {
        if self.ontology.replace(ontology).is_some() {
            return Err("set_ontology");
        }
        Ok(())
    }

    fn add_other_field(&mut self, name: String, value: BString) {
        self.other.push((name, value));
    }
}

enum AidField {
    Name(BString),
}

#[derive(Default)]
struct AidBuilder {
    name: Option<BString>,
}

impl AidBuilder {
    fn build(self) -> Result<Aid> {
        let Some(name) = self.name else {
            return Err("name");
        };

        name.to_str_lossy().parse().map_err(|e| {
            log::error!("Failed to parse aid name: {e}");
            "name"
        })
    }

    fn set_name(&mut self, name: BString) -> Result<()> {
        if self.name.replace(name).is_some() {
            return Err("set_name");
        }
        Ok(())
    }
}

peg::parser! {
    pub(super) grammar messsage<'a>() for crate::util::parsing::BStr<'a> {
        use alloc::vec;

        use bstr::ByteSlice;

        rule _ = [c if c.is_ascii_whitespace()]*

        pub rule message() -> Message
            = lbrace() _ p:performative() _ fs:(message_field() ** _) _ rbrace()
        {?
            let mut builder = MessageBuilder::default();
            for field in fs {
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

        rule message_field() -> MessageField
            = ":receiver" _ r:receiver() { MessageField::Receiver(r) }
            / ":language" _ l:word() { MessageField::Language(l) }
            / ":ontology" _ o:word()
                {?
                    let ontology = String::from_utf8(o.into())
                        .map_err(|_| "utf8 ontology string")?;
                        Ok(MessageField::Ontology(ontology))
                }
            / ":content" _ c:string() { MessageField::Content(c)}
            / ":" _ o:string()
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
