use alloc::string::ToString;

use bstr::{BString, ByteSlice};

use crate::agent::aid::Aid;
use crate::message::content::lang::sl::sl0_parser;
use crate::message::{Content, Message, Performative, Receiver};

type Result<T> = core::result::Result<T, &'static str>;

enum MessageField {
    Receiver(Receiver),
    Language(BString),
    Content(BString),
    Ontology(BString),
}

#[derive(Default)]
struct MessageBuilder {
    receiver: Option<Receiver>,
    language: Option<BString>,
    ontology: Option<BString>,
    content: Option<BString>,
}

impl MessageBuilder {
    fn build(self, performative: Performative) -> Result<Message> {
        let Some(receiver) = self.receiver else {
            return Err("receiver");
        };
        let ontology = self.ontology.map(|o| o.to_str_lossy().to_string());
        let Some(content) = self.content else {
            return Err("content");
        };

        let content = match self.language.as_ref().map(|l| l.as_slice()) {
            Some(b"fipa-sl0") => Content::Structured(
                sl0_parser::content(&content.as_bstr().into()).map_err(|e| {
                    log::error!("failed to parse content as sl0: {e}");
                    "content"
                })?,
            ),
            // TODO: Fix this when properly supporting sending bytes.
            Some(b"bytes") => Content::Bytes(hex::decode(content).map_err(|_| {
                log::error!("failed to parse bytes content from hex");
                "bytes-content"
            })?),
            None => Content::Other {
                kind: None,
                content: content.to_string(),
            },
            _ => todo!(),
        };

        Ok(Message {
            performative,
            sender: None,
            receiver,
            reply_to: None,
            ontology,
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

    fn set_ontology(&mut self, ontology: BString) -> Result<()> {
        if self.ontology.replace(ontology).is_some() {
            return Err("set_ontology");
        }
        Ok(())
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
                }
            }
            builder.build(p)
        }

        rule performative() -> Performative
            = w:word() {? w.to_str_lossy().parse().or(Err("performative")) }

        rule message_field() -> MessageField
            = ":receiver" _ r:receiver() { MessageField::Receiver(r) }
            / ":language" _ l:word() { MessageField::Language(l) }
            / ":ontology" _ o:word() { MessageField::Ontology(o) }
            / ":content" _ c:string() { MessageField::Content(c)}

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
