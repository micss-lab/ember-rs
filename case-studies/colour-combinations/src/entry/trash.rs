use ember::{
    Agent,
    behaviour::{Context, CyclicBehaviour},
};
use serde::{Deserialize, Serialize};

use self::ontology::TrashOntology;
use super::belt::Belt;

pub fn trasher_agent(belt: Belt) -> Agent<'static, Belt, ()> {
    Agent::new("trasher", belt).with_behaviour(TrashBehaviour)
}

#[derive(Serialize, Deserialize)]
pub struct TrashMessage;

pub mod ontology {
    use ember::{
        Aid,
        message::{Content, Message},
    };

    use super::TrashMessage;

    pub struct TrashOntology;

    impl TrashOntology {
        pub const fn name() -> &'static str {
            "Trasher-Ontology"
        }

        pub fn decode_message(message: Message) -> TrashMessage {
            let Content::Bytes(content) = message.content else {
                panic!("received incorrect content type");
            };
            postcard::from_bytes(&content).expect("failed to parse message content")
        }
    }

    impl TrashMessage {
        pub fn into_message(self) -> Message {
            use ember::message::{Performative, Receiver};
            Message {
                performative: Performative::Request,
                sender: None,
                receiver: Receiver::Single(Aid::local("trasher")),
                reply_to: None,
                ontology: Some(TrashOntology::name().into()),
                content: Content::Bytes(
                    postcard::to_allocvec(&self).expect("failed to serialize message"),
                ),
            }
        }
    }
}

struct TrashBehaviour;

impl CyclicBehaviour for TrashBehaviour {
    type AgentState = Belt;

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        let Some(TrashMessage) = ctx.receive_message(None).map(TrashOntology::decode_message)
        else {
            ctx.block_behaviour();
            return;
        };

        log::info!(
            "Trashing {:?} brick.",
            state
                .take_next()
                .expect("no item found when expecting to trash")
        )
    }

    fn is_finished(&self) -> bool {
        false
    }
}
