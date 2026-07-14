pub mod bit_efficient;
pub mod string;

mod builder {
    use alloc::collections::btree_map::BTreeMap;
    use alloc::string::String;
    use alloc::vec::Vec;

    use bstr::{BString, ByteSlice};

    use crate::agent::Aid;
    use crate::message::content::fipa_sl::Sl0Content;
    use crate::message::{Content, Message, Performative, Receiver};

    #[derive(Debug)]
    pub(super) enum MessageField {
        Receiver(Receiver),
        Language(BString),
        Content(BString),
        Ontology(String),
        Other(String, BString),
    }

    #[derive(Default)]
    pub(super) struct MessageBuilder {
        receiver: Option<Receiver>,
        language: Option<BString>,
        ontology: Option<String>,
        other: Vec<(String, BString)>,
        content: Option<BString>,
    }

    impl MessageBuilder {
        pub(super) fn build(self, performative: Performative) -> Result<Message, &'static str> {
            let content = if let Some(content) = self.content {
                Some(match self.language.as_ref().map(|l| l.as_slice()) {
                    Some(b"fipa-sl0") => Content::FipaSl0(
                        Sl0Content::try_from_sl(content.as_bstr()).map_err(|e| {
                            log::error!("failed to parse content as sl0: {e}");
                            "content"
                        })?,
                    ),
                    Some(b"bytes") => {
                        use base64ct::{Base64, Encoding};
                        Content::Bytes(
                            Base64::decode_vec(content.to_str_lossy().as_ref()).map_err(|e| {
                                log::error!("failed to parse bytes content from base64: {e}");
                                "content (bytes)"
                            })?,
                        )
                    }
                    None => {
                        log::warn!("message has no content language parameter");
                        Content::Other {
                            language: None,
                            content,
                        }
                    }
                    Some(l) => {
                        log::warn!(
                            "unrecognised content language `{}`, treating as opaque string",
                            bstr::BStr::new(l)
                        );
                        Content::Other {
                            language: Some(String::from_utf8(l.to_vec()).map_err(|e| {
                                log::error!("failed to parse message language param as utf-8: {e}");
                                "language"
                            })?),
                            content,
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

        pub(super) fn set_receiver(&mut self, receiver: Receiver) -> Result<(), &'static str> {
            if self.receiver.replace(receiver).is_some() {
                log::error!("duplicate field `receiver`");
                return Err("receiver");
            }
            Ok(())
        }

        pub(super) fn set_language(&mut self, language: BString) -> Result<(), &'static str> {
            if self.language.replace(language).is_some() {
                log::error!("duplicate field `language`");
                return Err("language");
            }
            Ok(())
        }

        pub(super) fn set_ontology(&mut self, ontology: String) -> Result<(), &'static str> {
            if self.ontology.replace(ontology).is_some() {
                log::error!("duplicate field `ontology`");
                return Err("ontology");
            }
            Ok(())
        }

        pub(super) fn set_content(&mut self, content: BString) -> Result<(), &'static str> {
            if self.content.replace(content).is_some() {
                log::error!("duplicate field `content`");
                return Err("content");
            }
            Ok(())
        }

        pub(super) fn add_other_field(&mut self, name: String, value: BString) {
            self.other.push((name, value));
        }
    }

    pub(super) enum AidField {
        Name(BString),
    }

    #[derive(Default)]
    pub(super) struct AidBuilder {
        name: Option<BString>,
    }

    impl AidBuilder {
        pub(super) fn build(self) -> Result<Aid, &'static str> {
            let Some(name) = self.name else {
                log::error!("missing aid field `name`");
                return Err("name");
            };

            name.to_str_lossy().parse().map_err(|e| {
                log::error!("failed to parse aid field `name`: {e}");
                "name"
            })
        }

        pub(super) fn set_name(&mut self, name: BString) -> Result<(), &'static str> {
            if self.name.replace(name).is_some() {
                log::error!("duplicate aid field `name`");
                return Err("name");
            }
            Ok(())
        }
    }
}
