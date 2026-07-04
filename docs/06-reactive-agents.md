# 6. Reactive Agents

Reactive agents are Ember's simpler, JADE-inspired agent architecture. A reactive agent is an object
that owns some state and schedules a set of **behaviours**; the container ticks the agent, and the
agent runs its behaviours. This page is the complete reference for the `ember-reactive` API.

## 6.1 The `ReactiveAgent`

```rust
use ember::agent::reactive::ReactiveAgent;

let agent = ReactiveAgent::new("worker", initial_state)
    .with_behaviour(BehaviourA)
    .with_behaviour(BehaviourB);
```

`ReactiveAgent::new(name, state)` creates an agent with a name and an initial **agent state** of any
type `S`. Behaviours are attached with `with_behaviour` (chainable) or `add_behaviour` (returns a
`BehaviourId` you can later use to remove it). The agent implements the core `Agent` trait, so it
drops straight into a `Container`.

Two associated types thread through every behaviour on an agent:

- **`AgentState`**: the shared, mutable state the agent owns; every behaviour receives `&mut S` in
  its `action`. This is how behaviours on the same agent share data.
- **`Event`**: the type of internal events behaviours can emit to coordinate with each other
  (especially inside complex behaviours like FSMs).

All behaviours on one agent must agree on the same `AgentState`. All behaviours under the same parent behaviour have to agree on the same `Event` type.

## 6.2 The behaviour model

Every behaviour ultimately implements the internal `Behaviour` trait, but you never write that
directly. Instead you implement one of the ergonomic trait aliases below, and Ember converts it into
a boxed behaviour via `IntoBehaviour`. The agent runs all its behaviours (in a parallel queue by
default) on each tick.

Each behaviour's `action` receives:

- `ctx: &mut Context<Self::Event>`: the behaviour context (see [§6.5](#65-the-behaviour-context)), and
- `state: &mut Self::AgentState`: the agent's shared state.

## 6.3 Simple behaviours

### `CyclicBehaviour`

Runs its `action` on **every tick**, forever (or until `is_finished` returns `true`).

```rust
impl CyclicBehaviour for Poller {
    type AgentState = ();
    type Event = ();

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        // runs each tick
    }

    fn is_finished(&self) -> bool { false }
}
```

### `OneShotBehaviour`

Runs its `action` **exactly once**, then finishes. Note `action` takes `&self` (it cannot mutate the
behaviour itself, only the agent state and context).

```rust
impl OneShotBehaviour for Init {
    type AgentState = ();
    type Event = ();

    fn action(&self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        // one-time setup
    }
}
```

### `TickerBehaviour`

Like `CyclicBehaviour`, but `action` runs only after a fixed **interval** has elapsed, using the
`ember-time` driver. Note, the interval is a lower-bound due to the cooperative nature of the framework.

```rust
impl TickerBehaviour for Heartbeat {
    type AgentState = ();
    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_secs(1)
    }

    fn action(&mut self, ctx: &mut Context<Self::Event>, state: &mut Self::AgentState) {
        log::info!("still alive");
    }

    fn is_finished(&self) -> bool { false }
}
```

## 6.4 Complex behaviours

Complex behaviours **compose other behaviours**. They implement `ComplexBehaviour`, which adds a
third associated type, `ChildEvent`, for the events of the behaviours they contain.

### `SequentialBehaviour`

Runs a list of child behaviours **one after another**, each to completion before the next starts.

### `ParallelBehaviour`

Runs a list of child behaviours **concurrently** (interleaved per tick). A `FinishStrategy` decides
when the whole parallel behaviour is considered finished: e.g. *never*, when *any* child finishes, or
when *all* do. (The `ReactiveAgent`'s own behaviour set is itself a parallel queue with the `Never`
strategy, which is why an agent keeps running as long as it lives.)

### `Fsm`: finite-state machine

Models each **state as a child behaviour** and drives transitions with typed triggers. Build one with
the `Fsm::builder()`:

```rust
impl FsmBehaviour<'static> for TrafficLight {
    type TransitionTrigger = Switch;

    fn fsm(&self) -> Fsm<'static, Self::AgentState, Self::TransitionTrigger, Self::ChildEvent> {
        let red    = RedLight::default().into_behaviour();
        let orange = OrangeLight::default().into_behaviour();
        let (start, green) = GreenLight::default().into_behaviour_with_id();

        Fsm::builder()
            .with_transition(red.id(), green.id(), Some(Switch))
            .with_transition(green.id(), orange.id(), Some(Switch))
            .with_transition(orange.id(), red.id(), Some(Switch))
            .with_behaviour(red, false)
            .with_behaviour(orange, false)
            .with_behaviour(green, false)
            .try_build(start)
            .unwrap()
    }
}
```

A state behaviour requests a transition by **emitting an event** (`FsmEvent`) carrying a
`TransitionTrigger`; the FSM looks up the matching transition and switches active state. See the
`traffic` and `behaviour_fsm` examples.

## 6.5 The behaviour `Context`

Inside `action`, `ctx: &mut Context<Event>` is your handle on the world. It **derefs to the
`Environment`**, so all messaging methods are available directly, plus behaviour-scheduling controls:

| Method                         | Effect                                                                     |
| ------------------------------ | -------------------------------------------------------------------------- |
| `ctx.send_message(msg)`        | Queue a message to send (via `Environment`)                                |
| `ctx.receive_message(filter)`  | Take a matching message from the inbox, or `None` (via `Environment`)      |
| `ctx.stop_platform()`          | Request the container to stop (via `Environment`)                          |
| `ctx.emit_event(event)`        | Emit an internal event of type `Event`                                     |
| `ctx.block_behaviour()`        | Block this behaviour until a new message arrives (see below)               |
| `ctx.reset_behaviour()`        | Reset this behaviour to its initial state                                  |
| `ctx.remove_behaviour(id)`     | Remove a behaviour (by `BehaviourId`) from the agent                       |
| `ctx.remove_agent()`           | Mark this agent for removal (self-termination / AMS deregistration)        |

### Blocking

A common pattern is a receiver that blocks when there is nothing to do, so it does not spin every
tick:

```rust
fn action(&mut self, ctx: &mut Context<Self::Event>, _: &mut Self::AgentState) {
    let Some(msg) = ctx.receive_message(None) else {
        ctx.block_behaviour();   // do not run the behaviour until a new message arrives
        return;
    };
    // handle msg
}
```

See the `block_behaviour` example.

## 6.6 A complete example

```rust
use ember::Container;
use ember::agent::reactive::ReactiveAgent;
use ember::agent::reactive::behaviour::{Context, TickerBehaviour};

struct Counter(u32);

impl TickerBehaviour for Counter {
    type AgentState = ();
    type Event = ();

    fn interval(&self) -> core::time::Duration {
        core::time::Duration::from_millis(500)
    }

    fn action(&mut self, _: &mut Context<Self::Event>, _: &mut Self::AgentState) {
        self.0 += 1;
        log::info!("count = {}", self.0);
    }

    fn is_finished(&self) -> bool { self.0 >= 5 }
}

fn main() {
    Container::new()
        .with_agent(ReactiveAgent::new("counter", ()).with_behaviour(Counter(0)))
        .start()
        .unwrap();
}
```

## 6.7 Next

- [BDI Agents](./07-bdi-agents.md): the reasoning-based alternative.
- [Examples](./11-examples.md): `behaviour_*`, `client_server`, `traffic`, `sensors`, and more.
