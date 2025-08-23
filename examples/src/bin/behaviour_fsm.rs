#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use ember::behaviour::fsm::{Fsm, FsmBehaviour, FsmEvent};
use ember_examples::setup_example;

setup_example!();

use alloc::string::String;
use core::cell::Cell;
use core::ptr::addr_of_mut;

use ember::behaviour::{
    ComplexBehaviour, Context, CyclicBehaviour, IntoBehaviourWithId, OneShotBehaviour,
    TickerBehaviour,
};
use ember::{Agent, Container};

static mut WORKER_MESSAGE: Option<WorkerMessage> = None;
static mut MANAGER_MESSAGE: Option<ManagerMessage> = None;

#[derive(Debug)]
struct WorkerMessage {
    task: String,
}

#[derive(Debug)]
enum ManagerMessage {
    Acknowledge,
    Finished,
}

fn manager() -> Agent<'static, (), ()> {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum ManagerTrigger {
        TaskSent,
        AcknowledgementReceived,
    }

    struct SendWworkerTask(Cell<String>);

    impl OneShotBehaviour for SendWworkerTask {
        type AgentState = ();

        type Event = FsmEvent<ManagerTrigger, ()>;

        fn action(&self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            log::trace!("Sending task to worker");
            unsafe {
                WORKER_MESSAGE = Some(WorkerMessage {
                    task: self.0.take(),
                });
            }
            ctx.emit_event(FsmEvent::Trigger(ManagerTrigger::TaskSent))
        }
    }

    struct ReceiveAcknowledgement;

    impl CyclicBehaviour for ReceiveAcknowledgement {
        type AgentState = ();

        type Event = FsmEvent<ManagerTrigger, ()>;

        fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            log::trace!("Waiting for acknowledgement.");

            let Some(message) = unsafe { &mut *addr_of_mut!(MANAGER_MESSAGE) }.take() else {
                return;
            };

            let ManagerMessage::Acknowledge = message else {
                panic!("Received unexpected message: {message:?}");
            };
            ctx.emit_event(FsmEvent::Trigger(ManagerTrigger::AcknowledgementReceived));
        }

        fn is_finished(&self) -> bool {
            false
        }
    }

    #[derive(Default)]
    struct ReceiveFinish {
        received: bool,
    }

    impl TickerBehaviour for ReceiveFinish {
        type AgentState = ();

        type Event = FsmEvent<ManagerTrigger, ()>;

        fn interval(&self) -> core::time::Duration {
            core::time::Duration::from_millis(500)
        }

        fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            let Some(message) = unsafe { &mut *addr_of_mut!(MANAGER_MESSAGE) }.take() else {
                return;
            };

            let ManagerMessage::Finished = message else {
                panic!("Received unexpected message: {message:?}");
            };

            self.received = true;
            log::info!("Manager received finished message.");
            log::info!("Exiting");
        }

        fn is_finished(&self) -> bool {
            self.received
        }
    }

    struct ManagerBehaviour;

    impl ComplexBehaviour for ManagerBehaviour {
        type AgentState = ();

        type Event = ();

        type ChildEvent = ();
    }

    impl FsmBehaviour<'_> for ManagerBehaviour {
        type TransitionTrigger = ManagerTrigger;

        fn fsm(&self) -> Fsm<'static, Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
            let send_worker_task =
                SendWworkerTask(Cell::new("Print this message".into())).into_behaviour_with_id();
            let receive_acknowledgement = ReceiveAcknowledgement.into_behaviour_with_id();
            let receive_finish = ReceiveFinish::default().into_behaviour_with_id();

            Fsm::builder()
                .with_behaviour(send_worker_task.1, false)
                .with_behaviour(receive_acknowledgement.1, false)
                .with_behaviour(receive_finish.1, true)
                .with_transition(
                    send_worker_task.0,
                    receive_acknowledgement.0,
                    Some(ManagerTrigger::TaskSent),
                )
                .with_transition(
                    receive_acknowledgement.0,
                    receive_finish.0,
                    Some(ManagerTrigger::AcknowledgementReceived),
                )
                .try_build(send_worker_task.0)
                .expect("fsm failed to build")
        }
    }

    Agent::new("manager", ()).with_behaviour(ManagerBehaviour)
}

fn worker() -> Agent<'static, (), ()> {
    static mut CURRENT_TASK: Option<String> = None;

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum WorkerTrigger {
        TaskReceived,
        SentAcknowledgement,
        PerformedTask,
    }

    struct ReceiveTask;

    impl TickerBehaviour for ReceiveTask {
        type AgentState = ();

        type Event = FsmEvent<WorkerTrigger, ()>;

        fn interval(&self) -> core::time::Duration {
            core::time::Duration::from_millis(500)
        }

        fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            log::trace!("Waiting for task from manager");
            let Some(message) = unsafe { &mut *addr_of_mut!(WORKER_MESSAGE) }.take() else {
                return;
            };
            unsafe {
                CURRENT_TASK = Some(message.task);
            }
            ctx.emit_event(FsmEvent::Trigger(WorkerTrigger::TaskReceived));
        }

        fn is_finished(&self) -> bool {
            false
        }
    }

    struct SendAcknowledgement;

    impl OneShotBehaviour for SendAcknowledgement {
        type AgentState = ();

        type Event = FsmEvent<WorkerTrigger, ()>;

        fn action(&self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            unsafe {
                MANAGER_MESSAGE = Some(ManagerMessage::Acknowledge);
            }
            ctx.emit_event(FsmEvent::Trigger(WorkerTrigger::SentAcknowledgement))
        }
    }

    struct PerformTask;

    impl OneShotBehaviour for PerformTask {
        type AgentState = ();

        type Event = FsmEvent<WorkerTrigger, ()>;

        fn action(&self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            log::info!("Performing task by printing given message");
            log::info!(
                "message: {}",
                unsafe { &mut *addr_of_mut!(CURRENT_TASK) }.take().unwrap()
            );
            ctx.emit_event(FsmEvent::Trigger(WorkerTrigger::PerformedTask));
        }
    }

    struct SendFinishedMessage;

    impl OneShotBehaviour for SendFinishedMessage {
        type AgentState = ();

        type Event = FsmEvent<WorkerTrigger, ()>;

        fn action(&self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
            log::info!("Worker finished performing task");
            unsafe {
                MANAGER_MESSAGE = Some(ManagerMessage::Finished);
            }
        }
    }

    struct WorkerBehaviour;

    impl ComplexBehaviour for WorkerBehaviour {
        type AgentState = ();

        type Event = ();

        type ChildEvent = ();
    }

    impl FsmBehaviour<'static> for WorkerBehaviour {
        type TransitionTrigger = WorkerTrigger;

        fn fsm(&self) -> Fsm<'static, Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
            let receive_task = ReceiveTask.into_behaviour_with_id();
            let send_acknowledgement = SendAcknowledgement.into_behaviour_with_id();
            let perform_task = PerformTask.into_behaviour_with_id();
            let send_finished = SendFinishedMessage.into_behaviour_with_id();

            Fsm::builder()
                .with_behaviour(receive_task.1, false)
                .with_behaviour(send_acknowledgement.1, false)
                .with_behaviour(perform_task.1, false)
                .with_behaviour(send_finished.1, true)
                .with_transition(
                    receive_task.0,
                    send_acknowledgement.0,
                    Some(WorkerTrigger::TaskReceived),
                )
                .with_transition(
                    send_acknowledgement.0,
                    perform_task.0,
                    Some(WorkerTrigger::SentAcknowledgement),
                )
                .with_transition(
                    perform_task.0,
                    send_finished.0,
                    Some(WorkerTrigger::PerformedTask),
                )
                .try_build(receive_task.0)
                .expect("fsm failed to build")
        }
    }

    Agent::new("worker", ()).with_behaviour(WorkerBehaviour)
}

fn example() {
    let container = Container::default()
        .with_agent(manager())
        .with_agent(worker());
    container.start().unwrap();
}
