#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
// Avoid unused imports on unsupported targets.
#![cfg_attr(target_os = "none", allow(unused))]

use alloc::format;
use core::str::FromStr;

use ember::behaviour::{Context, CyclicBehaviour, TickerBehaviour};
use ember::message::{Content, Message, MessageEnvelope, Performative, Receiver};
use ember::{Agent, Aid, Container};
use ember_examples::setup_example;

setup_example!();

const VALUES: [Metrics; 10] = [
    Metrics {
        temperature: 20.0,
        humidity: 50.0,
        light: 100.0,
    },
    Metrics {
        temperature: 22.0,
        humidity: 55.0,
        light: 110.0,
    },
    Metrics {
        temperature: 24.0,
        humidity: 60.0,
        light: 120.0,
    },
    Metrics {
        temperature: 26.0,
        humidity: 65.0,
        light: 130.0,
    },
    Metrics {
        temperature: 28.0,
        humidity: 70.0,
        light: 140.0,
    },
    Metrics {
        temperature: 26.0,
        humidity: 65.0,
        light: 130.0,
    },
    Metrics {
        temperature: 24.0,
        humidity: 60.0,
        light: 120.0,
    },
    Metrics {
        temperature: 22.0,
        humidity: 55.0,
        light: 110.0,
    },
    Metrics {
        temperature: 20.0,
        humidity: 50.0,
        light: 100.0,
    },
    Metrics {
        temperature: 18.0,
        humidity: 45.0,
        light: 90.0,
    },
];

// ======== Server ========

struct MetricsReceiver;

impl CyclicBehaviour for MetricsReceiver {
    type AgentState = ();

    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let Some(message) = ctx.receive_message(None) else {
            log::debug!("No message received. Waiting...");
            ctx.block_behaviour();
            return;
        };
        let metrics = Metrics::from(message.content);
        log::info!("Received metrics: {metrics:?}");
    }

    fn is_finished(&self) -> bool {
        false
    }
}

// ======== Client ========

struct ReadMetrics<V>(V);

impl<V> TickerBehaviour for ReadMetrics<V>
where
    V: Iterator<Item = Metrics>,
{
    type AgentState = ();

    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(5000)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        let metrics = self.0.next().expect("could not take measurement");
        log::debug!("Sending metrics.");
        ctx.send_message(metrics.into())
    }

    fn is_finished(&self) -> bool {
        false
    }
}

#[cfg(target_os = "none")]
fn example() {
    panic!("This example only works on targets with the standard library");
}

#[cfg(not(target_os = "none"))]
fn example() {
    let mut client_container = Container::default()
        .with_http(1338)
        // .with_agent(Agent::new("server").with_behaviour(MetricsReceiver))
        .with_agent(
            Agent::new("client", ()).with_behaviour(ReadMetrics(VALUES.into_iter().cycle())),
        );

    let mut server_container = Container::default()
        .with_http(1337)
        // .with_agent(Agent::new("server").with_behaviour(MetricsReceiver))
        .with_agent(Agent::new("server", ()).with_behaviour(MetricsReceiver));

    loop {
        server_container
            .poll()
            .expect("server container exited with an error");
        client_container
            .poll()
            .expect("client container exited with an error");
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    temperature: f32,
    humidity: f32,
    light: f32,
}

impl From<Content> for Metrics {
    fn from(content: Content) -> Self {
        let Content::Other { content, .. } = content else {
            panic!("message content invalid");
        };
        content.parse().expect("failed to parse content as metrics")
    }
}

impl FromStr for Metrics {
    type Err = ();

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let (temperature, content) = content
            .split_once(',')
            .expect("message should contain temperature");
        let (humidity, content) = content
            .split_once(',')
            .expect("message should contain humidity");
        let light = content;
        Ok(Self {
            temperature: temperature.parse().expect("invalid value for temperature"),
            humidity: humidity.parse().expect("invalid value for humidity"),
            light: light.parse().expect("invalid value for light"),
        })
    }
}

impl From<Metrics> for MessageEnvelope {
    fn from(value: Metrics) -> Self {
        MessageEnvelope::new(
            Aid::general("server", "localhost:1337"),
            Message {
                performative: Performative::Inform,
                sender: None,
                receiver: Receiver::Single(Aid::local("server")),
                reply_to: None,
                ontology: None,
                content: Content::Other {
                    kind: None,
                    content: format!("{},{},{}", value.temperature, value.humidity, value.light),
                },
            },
        )
    }
}
