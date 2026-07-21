# 7. BDI Agents

BDI (**Belief-Desire-Intention**) agents are Ember's reasoning-based agent architecture. Instead of
scheduling behaviours, a BDI agent maintains a **belief base** (what it knows), pursues **goals**
(what it wants), and executes **plans** (how it acts): the classic BDI model. Ember lets you write
BDI agents in an **AgentSpeak/Jason-inspired language** embedded directly in Rust source through the
`#[bdi_agent]` attribute macro.

This is the longest reference page in the docs; use the section list to navigate.

- [7.1 The BDI model in Ember](#71-the-bdi-model-in-ember)
- [7.2 Declaring an agent](#72-declaring-an-agent)
- [7.3 The AgentSpeak-like language](#73-the-agentspeak-like-language)
- [7.4 Beliefs](#74-beliefs)
- [7.5 Rules (derived beliefs)](#75-rules-derived-beliefs)
- [7.6 Goals and plans](#76-goals-and-plans)
- [7.7 Plan bodies, actions, and events](#77-plan-bodies-actions-and-events)
- [7.8 Built-in actions](#78-built-in-actions)
- [7.9 User-defined actions](#79-user-defined-actions)
- [7.10 Custom Rust types as terms](#710-custom-rust-types-as-terms)
- [7.11 Sensors and percepts](#711-sensors-and-percepts)
- [7.12 The reasoning cycle](#712-the-reasoning-cycle)
- [7.13 Inter-agent belief sharing](#713-inter-agent-belief-sharing)
  - [7.13.1 Sending to a variable receiver](#7131-sending-to-a-variable-receiver)

## 7.1 The BDI model in Ember

| Concept       | In Ember                                                                            |
| ------------- | ----------------------------------------------------------------------------------- |
| **Belief**    | A ground literal in the *belief base*, e.g. `at(agent, home)`.                       |
| **Rule**      | A logic-programming clause that *derives* beliefs, e.g. `cooling_active :- pump_active`. |
| **Desire/Goal** | An *achievement goal* `!g` the agent wants to bring about, or a *test goal* `?g`. |
| **Plan**      | A recipe: a triggering event, an optional context condition, and a body of steps.   |
| **Intention** | A plan the agent has committed to and is currently executing (on the intention stack). |
| **Action**    | A leaf step in a plan body: either a built-in (`.log`, `.send`, …) or a user-defined Rust method. |

The agent runs a **reasoning cycle** each container tick: it turns perceptions and messages into
events, selects applicable plans for those events, and advances its intentions one step at a time.

## 7.2 Declaring an agent

A BDI agent is a plain struct annotated with `#[bdi_agent]`. The `asl` argument holds the agent's
program; the struct body holds the agent's Rust-side state.

```rust
use ember::agent::bdi::{bdi_agent, bdi_actions};

#[bdi_agent(asl = {
    // beliefs, goals, and plans go here
    at(agent, home).
    !make_coffee.

    +!make_coffee : at(agent, kitchen)
      <- .log("info", "Coffee time!");
         .stop_platform().

    +!make_coffee
      <- !go_to(kitchen);
         !make_coffee.

    +!go_to(Dest)
      <- -at(agent, home);
         +at(agent, Dest).
})]
struct CoffeeAgent;

#[bdi_actions]
impl CoffeeAgent {}   // user-defined actions (none here)
```

The macro generates a `From<CoffeeAgent>` impl and an `into_agent()` method producing a fully-wired
`BdiAgent`. Add it to a container like any other agent:

```rust
Container::new()
    .with_agent(CoffeeAgent.into_agent())
    .start()
    .unwrap();
```

The agent's **name** is derived from the struct name, kebab-cased (`CoffeeAgent` → `coffee-agent`),
and it registers with the AMS as `coffee-agent@local` on start-up.

The macro accepts two arguments:

| Argument         | Required | Meaning                                                             |
| ---------------- | -------- | ------------------------------------------------------------------ |
| `asl = { … }`    | yes      | The agent program in the AgentSpeak-like language.                 |
| `percept_type = T` | no     | The percept type produced by the agent's sensors (defaults to `()`). |

## 7.3 The AgentSpeak-like language

A program is a sequence of **beliefs/rules**, **initial goals**, and **plans**, each terminated by a
`.`. The grammar is Prolog/AgentSpeak-flavoured:

```
program      ::= (belief | goal)* plan*

belief       ::= literal (":-" logical_expression)? "."
goal         ::= "!" literal "."
plan         ::= trigger_event (":" context)? "<-" body "."

literal      ::= "~"? functor ("(" term ("," term)* ")")?
term         ::= literal | Variable | Number | String
```

Lexical conventions (matching Prolog/AgentSpeak):

- **Functors / atoms** start with a lowercase letter (or `.` for built-in actions): `at`, `coffee_beans`.
- **Variables** start with an uppercase letter or `_`: `X`, `Dest`, `_Ignored`.
- **Numbers** are integer or floating-point literals; **strings** are `"double-quoted"`.
- `~` before a literal means **negation-as-failure** (NAF).

Because the program is parsed by a proc-macro over Rust tokens, malformed programs are **compile
errors** with a span pointing at the offending token.

## 7.4 Beliefs

A **belief** is a ground literal placed in the belief base at start-up:

```
at(agent, home).
at(coffee_machine, kitchen).
have(coffee_beans).
```

Beliefs are structured terms: `part(motor_1, nominal)` is a functor `part` with two atom arguments.
Arguments can nest arbitrarily: `route(from(base), to(factory))`.

Beliefs change at runtime through plan bodies (`+belief` / `-belief`), incoming messages, and
sensors. Adding or removing a belief can *trigger* plans (see [§7.6](#76-goals-and-plans)).

## 7.5 Rules (derived beliefs)

A belief can carry a **rule** (`:-`) making it *derived* rather than stored. The head holds whenever
the body is satisfiable against the current belief base: exactly like a Prolog clause:

```
pump_a_active     :- not pump_a_failed.
pump_b_active     :- not pump_b_failed.
cooling_active    :- pump_a_active | pump_b_active.
danger_level_high :- reactor_online & cooling_insufficient & pressure_high.
```

Rule bodies are logical expressions built from:

- literals and negated literals (`not literal` for NAF),
- conjunction `&`, disjunction `|`, and grouping `( … )`,
- relational comparisons (`<`, `>`, `<=`, `>=`, `==`, `!=`, `=` for unification),
- arithmetic (`+`, `-`, `*`, `/`) over numbers and variables.

Rules may share a head to express disjunction across clauses:

```
needs_evacuation :- danger_level_high & not reactor_meltdown_imminent.
needs_evacuation :- reactor_meltdown_imminent.
```

Variables in a rule body let a derived belief quantify over the belief base:

```
part(valve_b, degraded).
part(pump_c, broken).

system_critical :- part(Comp, broken).   // holds because pump_c is broken
system_warning  :- part(Comp, degraded). // holds because valve_b is degraded
```

Derived beliefs are evaluated on demand during unification; they never occupy storage and update
automatically as the underlying facts change. See the `bdi_rules` and `bdi_rule_body_vars` examples.

## 7.6 Goals and plans

### Initial goals

An initial goal `!g` seeds the agent with something to achieve at start-up:

```
!make_coffee.
```

This posts an *achievement-goal-addition* event, which the agent tries to satisfy by selecting a
plan.

### Plans

A plan has three parts:

```
+!make_coffee : at(agent, kitchen) & have(coffee_beans)   <-   body  .
└──── head ──┘  └────────────── context ──────────────┘      └ body ┘
```

- **Triggering event (head)**: what the plan responds to. The prefix combines a *trigger* and an
  optional *goal kind*:

  | Head prefix | Fires when …                                       |
  | ----------- | -------------------------------------------------- |
  | `+!g`       | achievement goal `g` is **added**                  |
  | `-!g`       | achievement goal `g` is **dropped/failed**         |
  | `+?g`       | test goal `g` is added                             |
  | `+b`        | belief `b` is **added**                            |
  | `-b`        | belief `b` is **removed**                          |

- **Context (`: …`)**: an optional guard: a logical expression (same grammar as rule bodies) that
  must hold against the current belief base for the plan to be *applicable*. Variables bound here are
  available in the body.

- **Body (`<- …`)**: a sequence of steps separated by `;` and ended with `.` (see [§7.7](#77-plan-bodies-actions-and-events)).

When an event fires, the agent gathers all plans whose head matches, keeps those whose context holds,
and selects one (the *first applicable* by default). Because plans are tried in order, put the most
specific plans first and a general fallback last:

```
+!go_to(Dest) : at(agent, Dest)                 // already there
  <- .log("info", "Already at destination").

+!go_to(Dest) : at(agent, From)                 // one hop
  <- move_location(From, Dest);
     -at(agent, From);
     +at(agent, Dest).
```

## 7.7 Plan bodies, actions, and events

A plan body is a `;`-separated sequence of steps. Each step is one of:

| Syntax          | Meaning                                                                 |
| --------------- | ----------------------------------------------------------------------- |
| `+belief`       | Add a belief (may trigger `+belief` plans)                              |
| `-belief`       | Remove a belief (may trigger `-belief` plans)                           |
| `!goal`         | Post a **subgoal**: pursue `goal` before continuing                    |
| `?goal`         | Test goal: query the belief base                                       |
| `.builtin(...)` | Invoke a **built-in action** (leading dot): see [§7.8](#78-built-in-actions) |
| `action(...)`   | Invoke a **user-defined action** (no leading dot): see [§7.9](#79-user-defined-actions) |

Subgoals make plans compose recursively, which is how multi-step behaviour is expressed:

```
+!deliver(Item, Dest)
  <- !go_to(warehouse);
     pickup(Item);
     !go_to(Dest);
     dropoff(Item);
     .stop_platform().
```

Here `!go_to(...)` recursively invokes the `go_to` plans (which may themselves plan a multi-hop
route), while `pickup` and `dropoff` are user-defined Rust actions. See the `bdi_logistics` example.

## 7.8 Built-in actions

Built-in actions are written with a **leading dot** and are provided by the runtime:

| Action                                   | Effect                                                                     |
| ---------------------------------------- | -------------------------------------------------------------------------- |
| `.log("level", term, …)`                 | Log the given terms at `level` (`"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"`). |
| `.stop_platform()`                       | Stop the whole container.                                                   |
| `.send(aid, "performative", lit)`        | Send belief `lit` to another agent; `aid` is a `"name@host"` string or a bound variable (see [§7.13](#713-inter-agent-belief-sharing)). |
| `.wait(millis)`                          | Suspend the current intention for at least `millis` milliseconds before continuing to the next step. `millis` must be an integer literal. Other intentions keep running while this one waits (see [§7.12](#712-the-reasoning-cycle)). |

Using an unknown `.builtin` is a compile error listing the valid built-ins.

## 7.9 User-defined actions

Any step in a plan body *without* a leading dot is a **user-defined action**: a method on the agent
struct, declared inside a `#[bdi_actions]` impl block. Its arguments come from the action's terms in
the plan, converted into your Rust types.

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromTerm)]
pub enum Location { Home, Kitchen }

#[bdi_actions]
impl CoffeeAgent {
    // called by `move_location(From, Dest)` in a plan body
    fn move_location(&mut self, from: Location, to: Location) {
        log::info!("Moving from {from:?} to {to:?}");
    }

    // called by `buy(coffee_beans)`
    fn buy(&mut self, item: Item) {
        log::info!("Buying {item:?}");
    }
}
```

Rules for action methods:

- The `#[bdi_actions]` macro generates an action enum (`CoffeeAgentAction`) and wires up dispatch.
- Each method takes `&mut self` (so actions can read and mutate the agent's Rust state) followed by
  one parameter per term in the call. Parameter types must implement [`FromTerm`](#710-custom-rust-types-as-terms).
- An action may optionally take a `&mut Context<...>` parameter to reach the environment directly
  (e.g. to call `context.stop_platform()` or send raw messages).
- Terms are matched positionally; the number of terms in the plan must match the method's arity.

The agent's own struct fields are the mutable state actions share:

```rust
#[bdi_agent(asl = { /* … */ })]
struct Robot { battery: u8 }

#[bdi_actions]
impl Robot {
    fn drive(&mut self, from: Location, to: Location) {
        self.battery = self.battery.saturating_sub(10);   // mutate agent state
    }
}
```

## 7.10 Custom Rust types as terms

BDI terms are `integer`, `float`, `string`, `variable`, or nested `literal`. To move data between the
symbolic world and your Rust types, Ember provides three derives:

### `FromTerm`: decode a term into a Rust value

Used for **action parameters** and for reading terms out of the belief base. The mapping is
functor-based and snake-cased:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromTerm)]
pub enum Room { LivingRoom, Bedroom }
// matches the atoms `living_room` and `bedroom`

#[derive(FromTerm)]
struct Position { x: f32, y: f32 }
// matches `position(X, Y)` with two numeric arguments
```

Use `#[ember(transparent)]` on a single-field tuple struct to decode straight from the inner type
(no wrapping functor), which is handy for newtypes over `String`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, FromTerm)]
#[ember(transparent)]
pub struct Item(String);   // decodes any string term into Item
```

### `IntoLiteral`: encode a Rust value into a belief

The inverse: turns a value into a `Literal` (functor = snake-cased type/variant name, fields =
arguments). Primarily used by sensors ([§7.11](#711-sensors-and-percepts)):

```rust
#[derive(IntoLiteral)]
struct SensorReading { temperature: f32 }
// produces the belief `sensor_reading(<temperature>)`
```

### `Percept`: mark a type as a perception

See the next section for how percepts are consumed. By default, `#[derive(Percept)]` makes the type
become a single *added* belief via its `IntoLiteral` impl — but this is configurable through the
shared `#[ember(...)]` helper attribute (used by `FromTerm` above too), so a percept can express much
more than "add one belief":

- `add` / `add(<expr>)` — emit an *addition* of the belief produced by `<expr>` (any expression whose
  type implements `IntoLiteral`); `<expr>` defaults to `self` when omitted.
- `remove` / `remove(<expr>)` — same, but a *deletion*.
- `ignore` — this item/variant produces no belief at all.
- Several actions in one list emit several `(Trigger, Literal)` tuples for that single percept
  instance (e.g. "add belief A and remove belief B" as one state-transition event).

The attribute can be placed on the container (a struct, or as an enum-wide default) and/or on
individual enum variants, where a variant's own list overrides the container default. With nothing
specified anywhere, behavior is unchanged from a plain `#[derive(Percept)]`: a single `add(self)`.

```rust
// A percept that always removes a belief instead of adding one:
#[derive(IntoLiteral, Percept)]
#[ember(remove)]
struct GoneReading { sensor_id: u32 }

// An enum where different variants add vs. remove the same belief:
#[derive(IntoLiteral)]
struct DoorOpen;

#[derive(Percept)]
enum Door {
    #[ember(add(DoorOpen))]
    Opened,
    #[ember(remove(DoorOpen))]
    Closed,
}

// Regardless of variant, always the same belief (container-level default applies to every variant
// that doesn't override it):
#[derive(Percept)]
#[ember(add(PowerOnMarker))]
enum PowerState { On, Off, Standby }

// A custom belief built from the variant's own fields, or several actions in one shot:
#[derive(Percept)]
enum Event {
    #[ember(add(Alarm { since: opened_at }))]
    Opened { opened_at: u64, reason: String },
    #[ember(add(NewState(value)), remove(OldState))]
    Transition { value: i32 },
}
```

Named fields are referenced by their declared name, as above. Unnamed (tuple) fields are referenced
as `_0`, `_1`, … — the same convention `derive_more`'s `Display` derive uses for tuple fields in
format arguments — rather than a bare number, so there's no ambiguity between a field reference and
an actual literal value in the expression:

```rust
#[derive(IntoLiteral)]
struct ClearanceTimer(f32);

#[derive(Percept)]
enum DoorSensor {
    #[ember(add(ClearanceTimer(_0)))]
    Clearance(f32),
}
```

Every derive that reads configuration through `#[ember(...)]` owns a namespace key equal to its own
snake_case name (`percept` for `Percept`, `from_term` for `FromTerm`). The flat spellings above
(`#[ember(add(..))]`, `#[ember(transparent)]`) are shorthand for the common case where only one such
derive is in play on a given item; if you stack several `ember`-aware derives on the same struct or
enum and need to be explicit about which configuration belongs to which, wrap each one in its
namespace:

```rust
#[derive(IntoLiteral, Percept, FromTerm)]
#[ember(from_term(transparent), percept(remove))]
struct Gone(SomeInner);
```

If a derive does not fit at all, you can implement `FromTerm` / `IntoLiteral` / `Percept` by hand: the
`bdi_coffee` example does exactly this.

## 7.11 Sensors and percepts

Sensors connect the agent to the physical world. A **perceptor** is polled each tick and may return a
**percept**, which the agent folds into its belief base.

```rust
use ember::agent::bdi::sensor::{Percept, Perceptor};

// The perception type: becomes a belief when produced.
#[derive(IntoLiteral, Percept)]
struct SensorReading { temperature: f32 }

// The sensor itself.
struct Thermometer(/* pin, handle, … */);

impl Perceptor for Thermometer {
    type Percept = SensorReading;

    fn percept(&mut self) -> Option<Self::Percept> {
        Some(SensorReading { temperature: read_pin() })
    }
}
```

Attach sensors when building the agent, and declare the percept type on the macro so the belief types
line up:

```rust
#[bdi_agent(percept_type = SensorReading, asl = { /* … */ })]
struct CoffeeAgent;

Container::new()
    .with_agent(CoffeeAgent.into_agent().with_sensor(Thermometer(/* … */)))
    .start()
    .unwrap();
```

Each tick, every sensor is polled; each returned percept is converted (via `Percept::into_beliefs`)
into one or more `(Trigger, Literal)` pairs that are asserted into (or removed from) the belief base,
potentially triggering plans. The default percept type is `()`, which produces no beliefs: use it
for agents that only reason and communicate.

## 7.12 The reasoning cycle

Each container tick, a BDI agent's `update`:

1. Runs the FIPA/AMS registration component (once, on start-up).
2. **Polls sensors** and asserts/retracts the resulting beliefs.
3. **Reads inbound `ember-bdil` messages** and turns them into belief events ([§7.13](#713-inter-agent-belief-sharing)).
4. **Selects the next event** from the event queue and finds an applicable plan for it, pushing a new
   intention (or extending an existing one for a subgoal).
5. **Advances the intention stack one step**, resolving variable bindings and collecting the actions
   and events that step produced.
6. **Executes those actions** (built-in and user), which may add beliefs, post goals, send messages,
   or stop the platform.
7. **Queues newly generated events** for subsequent ticks.

The agent reports itself **finished** to the container when it has no remaining intentions, so a BDI
agent that has achieved all its goals lets the platform move on (and, if it was the last agent,
allows the container to idle or stop).

Some actions need more than one tick to finish — `.wait` is the built-in example. While such an
action is still in progress, its intention doesn't advance to its next step (so later steps in the
same plan wait for it), but other intentions continue to be scheduled normally, one step per tick, in
the meantime.

## 7.13 Inter-agent belief sharing

BDI agents share belief state over the network using the **`ember-bdil`** content language (see
[The `ember-bdil` Content Language](./08-bdil.md) for the wire format). From the DSL you send a
belief with `.send`:

```rust
#[bdi_agent(asl = {
    !startup.
    +!startup
      <- .send("receiver-agent@local", "inform", resource(water)).
})]
struct SenderAgent;
```

`.send(aid, performative, literal)` takes:

- **`aid`**: the receiver, either a literal `"name@host"` string (or `"name@local"`), or a bound
  variable. A literal address is syntactically validated at compile time; a malformed AID is a
  compile error.
- **`performative`**: `"inform"` (assert the belief on the receiver) or `"disconfirm"` (retract it).
- **`literal`**: the belief to transmit.

On the receiving side, the incoming belief arrives wrapped in `message(...)`, so the sender's
`resource(water)` is delivered as the belief `message(resource(water))`. React to it with an ordinary
belief-addition plan:

```rust
#[bdi_agent(asl = {
    +message(resource(X))
      <- .log("info", "Received resource: ", X);
         .stop_platform().
})]
struct ReceiverAgent;
```

See the `bdi_send` example for a runnable version.

### 7.13.1 Sending to a variable receiver

Instead of a literal address, `aid` may be a variable already bound (e.g. by the triggering event or
an earlier step in the plan body). This is useful when the destination is only known at runtime —
for example, replying to whoever sent a registration request:

```rust
#[bdi_agent(asl = {
    +register(Addr)
      <- .send(Addr, "inform", ack).
})]
struct RegistrarAgent;
```

The variable must be bound, at the point `.send` executes, to a string in the same `"name@host"` (or
`"name@local"`) format as a literal AID — there is no compile-time validation in this case, since the
address is only known at runtime. If the variable is unbound, or bound to a value that cannot be
parsed as an AID, the send is skipped and an error is logged instead of stopping the plan.

## 7.14 Next

- [The `ember-bdil` Content Language](./08-bdil.md): the wire format for belief sharing.
- [Examples](./11-examples.md): `bdi_coffee`, `bdi_coffee_asl`, `bdi_rules`, `bdi_logistics`,
  `bdi_smart_home`, `bdi_send`.
