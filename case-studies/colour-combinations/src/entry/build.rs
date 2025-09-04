use ember::{
    Agent,
    behaviour::{Context, CyclicBehaviour},
};
use serde::{Deserialize, Serialize};

use self::ontology::BuilderOntology;
use super::{Colour, belt::Belt};

pub fn builder_agent(belt: Belt) -> Agent<'static, Belt, ()> {
    Agent::new("builder", belt).with_behaviour(BuildBehaviour::default())
}

#[derive(Serialize, Deserialize)]
pub struct BuildMessage;

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message},
    };

    use super::BuildMessage;

    pub struct BuilderOntology;

    impl BuilderOntology {
        pub const fn name() -> &'static str {
            "Builder-Ontology"
        }

        pub fn decode_message(message: Message) -> BuildMessage {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse message content")
        }
    }

    impl BuildMessage {
        pub fn into_message(self) -> Message {
            use ember::message::{Performative, Receiver};
            Message {
                performative: Performative::Request,
                sender: None,
                receiver: Receiver::Single(Aid::local("builder")),
                reply_to: None,
                ontology: Some(BuilderOntology::name().into()),
                content: Content::Bytes(
                    postcard::to_allocvec(&self).expect("failed to serialize message"),
                ),
            }
        }
    }
}

#[derive(Default)]
struct BuildBehaviour {
    stored: Option<Colour>,
}

impl BuildBehaviour {
    fn store(&mut self, colour: Colour, state: &mut Belt) {
        match self.stored.take() {
            Some(stored) => {
                let score = state.made_combination(stored, colour);
                log::info!("Storing {:?} brick on top", colour);
                log::info!(
                    "Combining {:?} and {:?} for a score of {}",
                    stored,
                    colour,
                    score
                );
            }
            None => {
                log::info!("Storing {:?} brick", colour);
                self.stored = Some(colour)
            }
        }
    }
}

impl CyclicBehaviour for BuildBehaviour {
    type AgentState = Belt;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(BuildMessage) = ctx
            .receive_message(None)
            .map(BuilderOntology::decode_message)
        else {
            ctx.block_behaviour();
            return;
        };

        self.store(
            state
                .take_next()
                .expect("no item found when expecting to build"),
            state,
        );
    }

    fn is_finished(&self) -> bool {
        false
    }
}
