#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use alloc::format;
use core::str::FromStr;

use ember::Container;
use ember::agent::Aid;
use ember::agent::reactive::ReactiveAgent;
use ember::agent::reactive::behaviour::{Context, CyclicBehaviour, TickerBehaviour};
use ember::message::{Content, Message, Performative, Receiver};

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
        let metrics = Metrics::from(message.content.expect("message has no content"));
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

fn example() {
    let container = Container::default()
        .with_agent(ReactiveAgent::new("server", ()).with_behaviour(MetricsReceiver))
        .with_agent(
            ReactiveAgent::new("client", ())
                .with_behaviour(ReadMetrics(VALUES.into_iter().cycle())),
        );

    container.start().expect("container exited with error");
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
        core::str::from_utf8(&content)
            .expect("content should be valid utf-8")
            .parse()
            .expect("failed to parse content as metrics")
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

impl From<Metrics> for Message {
    fn from(value: Metrics) -> Self {
        Message {
            performative: Performative::Inform,
            receiver: Some(Receiver::Single(Aid::local("server"))),
            ontology: None,
            other: None,
            content: Some(Content::Other {
                language: None,
                content: format!("{},{},{}", value.temperature, value.humidity, value.light).into(),
            }),
        }
    }
}
