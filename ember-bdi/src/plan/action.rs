use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;

use bstr::BString;
use derive_where::derive_where;
use log::{Level, log};

use ember_time::{Duration, Instant};
use ember_util::cmp::TotalCmpF32;

use ember_core::agent::Aid;
use ember_core::environment::Environment;
use ember_core::message::content::ember_bdil::BdilContent;
use ember_core::message::{Content, Message, Performative, Receiver};

use crate::bindings::{AliasMap, BindingLookup, Bindings, OwnedBindings};
use crate::context::{Context, PureContext};
use crate::event::Trigger;
use crate::knowledge::base::KnowledgeBase;
use crate::knowledge::query::IntoQuery;
use crate::literal::Literal;
use crate::plan::{GoalKind, TriggeringEvent};
use crate::resolve::Resolve;
use crate::term::Term;
use crate::term::owned::composite::{VariableOrLiteral, VariableOrReceiver};
use crate::term::view::TermView;
use crate::variable::Variable;

use super::QueryFormula;

pub trait Execute: Sized {
    type State;

    /// The user action that is stored in the context. If you are unsure about this type, use
    /// `Self`.
    type UserAction;

    /// Executes the action returning `None` if it has finshed and a new action state if the action
    /// is to be ran again.
    fn execute<'b, B>(
        self,
        bindings: B,
        context: &mut Context<Self::UserAction>,
        knowledge: &KnowledgeBase,
        state: &mut Self::State,
    ) -> ExecuteResult<'b, Self>
    where
        B: BindingLookup;

    /// Should the intention this action is fired by wait for the action to complete before
    /// continuing.
    fn should_block_intention(&self) -> bool {
        true
    }
}

pub enum ExecuteResult<'b, S> {
    Pending(S),
    Done(Option<Bindings<'b>>),
}

impl<'b, S> ExecuteResult<'b, S> {
    pub fn map<U>(self, f: impl FnOnce(S) -> U) -> ExecuteResult<'b, U> {
        match self {
            ExecuteResult::Pending(s) => ExecuteResult::Pending(f(s)),
            ExecuteResult::Done(bindings) => ExecuteResult::Done(bindings),
        }
    }
}

#[derive_where(Debug, PartialEq, Eq)]
#[derive(Clone)]
pub enum Action<A> {
    Builtin(BuiltinAction),
    User(#[derive_where(skip)] A),
}

impl<A, S> Execute for Action<A>
where
    A: Execute<State = S, UserAction = A>,
{
    type State = S;

    type UserAction = A;

    fn execute<'b, B>(
        self,
        bindings: B,
        context: &mut Context<Self::UserAction>,
        knowledge: &KnowledgeBase,
        state: &mut Self::State,
    ) -> ExecuteResult<'b, Self>
    where
        B: BindingLookup,
    {
        match self {
            Action::Builtin(action) => action
                .execute(bindings, context, knowledge)
                .map(Action::Builtin),
            Action::User(action) => action
                .execute(bindings, context, knowledge, state)
                .map(Action::User),
        }
    }

    fn should_block_intention(&self) -> bool {
        match self {
            Action::Builtin(action) => action.should_block_intention(),
            Action::User(a) => a.should_block_intention(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PendingAction<A> {
    action: Action<A>,
    bindings: OwnedBindings,
}

impl<A> PendingAction<A> {
    pub(crate) fn new(action: Action<A>, bindings: OwnedBindings) -> Self {
        Self { action, bindings }
    }
}

impl<S, A> PendingAction<A>
where
    A: Execute<State = S, UserAction = A>,
{
    pub(crate) fn execute(
        self,
        context: &mut Context<A>,
        knowledge: &KnowledgeBase,
        state: &mut S,
    ) -> ExecuteResult<'static, Self> {
        let Self { action, bindings } = self;

        match action.execute(&bindings, context, knowledge, state) {
            ExecuteResult::Pending(action) => ExecuteResult::Pending(Self { action, bindings }),
            ExecuteResult::Done(None) => ExecuteResult::Done(None),
            ExecuteResult::Done(Some(bindings)) => ExecuteResult::Done(Some(bindings)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltinAction {
    /// A side-effect-free action: reads something and binds a variable, nothing else. The only
    /// kind of built-in action allowed in a plan's context guard.
    Pure(PureAction),
    /// A built-in action that touches the environment, sends a message, or otherwise has an
    /// effect beyond producing bindings. Body-only; never valid in a context guard.
    Impure(ImpureAction),
}

impl BuiltinAction {
    pub fn at(delay: core::time::Duration, goal: Literal) -> Self {
        BuiltinAction::Impure(ImpureAction::At(AtState {
            start: None,
            delay: ember_time::from_core_duration(delay),
            goal,
        }))
    }

    pub fn wait(interval: core::time::Duration) -> Self {
        BuiltinAction::Impure(ImpureAction::Wait(WaitState {
            start: None,
            interval: ember_time::from_core_duration(interval),
        }))
    }

    pub(crate) fn execute<'b, B, A>(
        self,
        bindings: B,
        context: &mut Context<A>,
        knowledge: &KnowledgeBase,
    ) -> ExecuteResult<'b, Self>
    where
        B: BindingLookup,
    {
        match self {
            BuiltinAction::Pure(action) => {
                ExecuteResult::Done(action.evaluate(bindings, &context.pure))
            }
            BuiltinAction::Impure(action) => action
                .execute(bindings, context, knowledge)
                .map(BuiltinAction::Impure),
        }
    }

    fn should_block_intention(&self) -> bool {
        match self {
            BuiltinAction::Pure(_) => true,
            BuiltinAction::Impure(action) => !matches!(action, ImpureAction::At(_)),
        }
    }
}

/// Built-in actions with no side effects, usable both in a plan body and in a plan's context
/// guard.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PureAction {
    /// Binds the current monotonic time in milliseconds to the given variable.
    Now(Variable),
    /// Binds the agent's own fully-qualified AID (`name@platform`) to the given variable.
    // TODO: Use unification instead of just binding allowing this action to be used as a check,
    // not only a fetch.
    Me(Variable),
    /// Binds `List` with `Item` appended to the given variable.
    Append(Term, Term, Variable),
    /// Whether `Item` structurally equals some element of `List`.
    Member(Term, Term),
}

impl PureAction {
    pub(crate) fn evaluate(
        &self,
        bindings: impl BindingLookup,
        pure_context: &PureContext,
    ) -> Option<Bindings<'static>> {
        Some(match self {
            PureAction::Now(variable) => {
                let millis = ember_time::now().duration_since_epoch().to_millis();
                Bindings::new(
                    [(
                        variable.id,
                        Some(TermView::Number(TotalCmpF32::from(millis as f32))),
                    )],
                    AliasMap::empty(),
                )
            }
            PureAction::Me(variable) => {
                let aid =
                    Aid::local(pure_context.agent_name.as_ref().clone().into_owned()).to_string();
                Bindings::new(
                    [(
                        variable.id,
                        Some(TermView::String(BString::from(aid).into())),
                    )],
                    AliasMap::empty(),
                )
            }
            PureAction::Append(list, item, variable) => {
                let items = match list.resolve_as_view(&bindings) {
                    Ok(TermView::List(items)) => items,
                    _ => {
                        // TODO: Like in prolog, solve this using lazy evaluation.
                        log::error!(".append: first argument did not resolve to a list");
                        return None;
                    }
                };
                let Ok(item) = item.resolve_as_view(&bindings) else {
                    log::error!(".append: failed to resolve item argument");
                    return None;
                };
                let items = items
                    .iter()
                    .chain(core::iter::once(&item))
                    .map(TermView::to_owned_view)
                    .collect();
                let view = TermView::List(items);
                Bindings::new([(variable.id, Some(view))], AliasMap::empty())
            }
            PureAction::Member(item, list) => {
                let item = item.resolve_as_view(&bindings).ok()?;
                let TermView::List(items) = list.resolve_as_view(&bindings).ok()? else {
                    // TODO: Like in prolog, solve this using lazy evaluation.
                    log::error!(".member: first argument did not resolve to a list");
                    return None;
                };
                if !items.contains(&item) {
                    return None;
                }
                Bindings::empty()
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackKind {
    OnSuccess,
    OnRetry,
    OnFailure,
    OnComplete,
}

/// Creates a request message sent back to the agent.
fn fire_goal(goal: Literal, my_aid: Aid, environment: &mut Environment) {
    environment.send_message(Message {
        performative: Performative::Request,
        receiver: Some(Receiver::Single(my_aid)),
        ontology: None,
        other: None,
        content: Some(Content::Bdil(BdilContent::Literal(goal.into()))),
    });
}

/// Built-in actions that touch the environment, send a message, or otherwise have an effect
/// beyond producing bindings. Body-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImpureAction {
    /// Log information to the stdout with the given log level.
    Log(Level, Box<[Term]>),
    /// Terminate the execution of the platform the agent is running on.
    StopPlatform,
    /// Send a belief update to another agent.
    SendLiteral(
        VariableOrReceiver,
        Trigger,
        VariableOrLiteral,
        Box<[(CallbackKind, VariableOrLiteral)]>,
    ),
    /// Halt the execution of an agents intention until the interval is finished. Construct this
    /// variant with the `[wait](WaitState::wait)` member function.
    Wait(WaitState),
    /// Spawn a new intention for all possible bindings resulting from unification with the pattern.
    Forall {
        query: QueryFormula,
        goal: VariableOrLiteral,
    },
    /// Raises an achievement-goal event once `delay` has elapsed. Construct this variant with
    /// the `[at](BuiltinAction::at)` member function.
    At(AtState),
}

impl ImpureAction {
    fn execute<'b, B, A>(
        self,
        bindings: B,
        context: &mut Context<A>,
        knowledge: &KnowledgeBase,
    ) -> ExecuteResult<'b, Self>
    where
        B: BindingLookup,
    {
        use ImpureAction::*;
        match self {
            Log(level, terms) => {
                match terms
                    .into_iter()
                    .map(|t| t.resolve(&bindings).map(|t| t.to_string()))
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(terms) => log!(level, "{terms:?}"),
                    Err(_) => log::error!("failed to resolve log arguments"),
                }
                ExecuteResult::Done(None)
            }
            StopPlatform => {
                context.stop_platform();
                ExecuteResult::Done(None)
            }
            SendLiteral(receiver, trigger, literal, callbacks) => {
                let performative = match trigger {
                    Trigger::Addition => Performative::Inform,
                    Trigger::Deletion => Performative::NotUnderstood,
                };
                let receiver = match receiver.resolve(&bindings) {
                    Ok(VariableOrReceiver::Receiver(r)) => r,
                    Ok(_) => {
                        log::error!("failed to resolve .send arguments");
                        return ExecuteResult::Done(None);
                    }
                    Err(_) => {
                        log::error!("failed to parse receiver");
                        return ExecuteResult::Done(None);
                    }
                };

                let literal = match literal.resolve(&bindings) {
                    Ok(VariableOrLiteral::Literal(l)) => l,
                    _ => {
                        log::error!(".send literal arg did not resolve to a literal");
                        return ExecuteResult::Done(None);
                    }
                };
                let my_aid = Aid::local(context.pure.agent_name.as_ref().clone().into_owned());
                let mut builder = context.send_message(Message {
                    performative,
                    receiver: Some(receiver),
                    ontology: None,
                    other: None,
                    content: Some(Content::Bdil(BdilContent::Literal(literal.into()))),
                });
                for (kind, goal) in Vec::from(callbacks) {
                    let goal = match goal.resolve(&bindings) {
                        Ok(VariableOrLiteral::Literal(l)) => l,
                        _ => {
                            log::error!("failed to resolve .send callback goal to a literal");
                            continue;
                        }
                    };
                    let my_aid = my_aid.clone();
                    builder = match kind {
                        CallbackKind::OnSuccess => builder
                            .on_success(move |environment| fire_goal(goal, my_aid, environment)),
                        CallbackKind::OnFailure => builder
                            .on_failure(move |environment| fire_goal(goal, my_aid, environment)),
                        CallbackKind::OnComplete => builder
                            .on_complete(move |environment| fire_goal(goal, my_aid, environment)),
                        CallbackKind::OnRetry => builder.on_retry(move |_attempt, environment| {
                            fire_goal(goal.clone(), my_aid.clone(), environment)
                        }),
                    };
                }
                ExecuteResult::Done(None)
            }
            Wait(state) => state.poll().map(Wait),
            At(state) => state.poll(bindings, context).map(At),
            Forall { query, goal } => {
                let pure_context = context.pure.clone();
                let mut query = query.into_query(knowledge, &pure_context);
                while let Some(bindings) = query.next_bindings(Some(&bindings.as_bindings())) {
                    let goal = match goal.clone().resolve(&bindings) {
                        Ok(VariableOrLiteral::Literal(l)) => l,
                        _ => {
                            log::error!(
                                "failed to resolve goal in forall body with queried bindings to a literal"
                            );
                            continue;
                        }
                    };
                    context.emit_event(
                        TriggeringEvent {
                            trigger: Trigger::Addition,
                            goal: Some(GoalKind::Achieve),
                            event: goal,
                        },
                        None,
                    );
                }
                ExecuteResult::Done(None)
            }
        }
    }
}

/// State for the `.at` built-in action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtState {
    start: Option<Instant>,
    delay: Duration,
    goal: Literal,
}

impl AtState {
    fn poll<'b, B, A>(self, bindings: B, context: &mut Context<A>) -> ExecuteResult<'b, Self>
    where
        B: BindingLookup,
    {
        let Self { start, delay, goal } = self;

        let Some(start) = start else {
            let armed = Self {
                start: Some(ember_time::now()),
                delay,
                goal,
            };

            return ExecuteResult::Pending(armed);
        };

        if ember_time::now() - start < delay {
            return ExecuteResult::Pending(Self {
                start: Some(start),
                delay,
                goal,
            });
        }

        match goal.resolve(bindings) {
            Ok(resolved) => context.emit_event(
                TriggeringEvent {
                    trigger: Trigger::Addition,
                    goal: Some(GoalKind::Achieve),
                    event: resolved,
                },
                None,
            ),
            Err(_) => log::error!("failed to resolve goal in .at"),
        }

        ExecuteResult::Done(None)
    }
}

/// State for the `.wait` built-in action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitState {
    start: Option<Instant>,
    interval: Duration,
}

impl WaitState {
    fn poll(self) -> ExecuteResult<'static, Self> {
        let Self { start, interval } = self;
        let Some(start) = start else {
            return ExecuteResult::Pending(Self {
                start: Some(ember_time::now()),
                interval,
            });
        };

        if ember_time::now() - start >= interval {
            return ExecuteResult::Done(None);
        }
        ExecuteResult::Pending(Self {
            start: Some(start),
            interval,
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use ember_core::agent::Aid;
    use ember_core::message::Receiver;

    use crate::resolve::ResolveFailure;
    use crate::term::conversion::{ConversionError, FromTermError};
    use crate::term::view::TermView;
    use crate::testing::{
        bindings, list, new_context_with_environment, new_context_without_environment, string,
        variable, variable_term,
    };

    use super::*;

    #[test]
    fn test_log_resolves_bound_variables_and_completes_immediately_without_side_effects() {
        // SAFETY: `.log` never touches the environment.
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let knowledge = KnowledgeBase::default();
        let var = variable();
        let room_value = string("kitchen");
        let bindings = bindings(vec![(var.clone(), room_value.as_view())]);

        let action = BuiltinAction::Impure(ImpureAction::Log(
            Level::Info,
            vec![variable_term(&var)].into_boxed_slice(),
        ));

        let result = action.execute(&bindings, &mut context, &knowledge);

        assert!(matches!(result, ExecuteResult::Done(None)));
        assert!(context.events.is_empty());
        assert!(context.actions.is_empty());
    }

    #[test]
    fn test_log_completes_immediately_even_when_a_term_fails_to_resolve() {
        // SAFETY: `.log` never touches the environment.
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let knowledge = KnowledgeBase::default();
        let bindings = bindings(vec![]);

        let action = BuiltinAction::Impure(ImpureAction::Log(
            Level::Info,
            vec![variable_term(&variable())].into_boxed_slice(),
        ));

        let result = action.execute(&bindings, &mut context, &knowledge);

        assert!(matches!(result, ExecuteResult::Done(None)));
    }

    #[test]
    fn test_stop_platform_sets_the_environment_stop_flag_and_completes_immediately() {
        let mut context: Context<()> = new_context_with_environment();
        let bindings = bindings(vec![]);
        let knowledge = KnowledgeBase::default();

        let result = BuiltinAction::Impure(ImpureAction::StopPlatform).execute(
            &bindings,
            &mut context,
            &knowledge,
        );

        assert!(matches!(result, ExecuteResult::Done(None)));
        assert!(
            context.stop_platform,
            ".stop_platform must set the environment's stop flag"
        );
    }

    #[test]
    fn test_wait_stays_pending_until_interval_elapses_then_completes() {
        // SAFETY: `.wait` never touches the environment.
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let bindings = bindings(vec![]);
        let knowledge = KnowledgeBase::default();

        let action = BuiltinAction::wait(core::time::Duration::from_millis(0));

        let ExecuteResult::Pending(action) = action.execute(&bindings, &mut context, &knowledge)
        else {
            panic!("the first poll only records the start time and must stay pending");
        };

        let result = action.execute(&bindings, &mut context, &knowledge);
        assert!(
            matches!(result, ExecuteResult::Done(None)),
            "a zero-length wait must complete on its second poll"
        );
    }

    #[test]
    fn test_now_binds_current_millis_to_the_given_variable() {
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let bindings = bindings(vec![]);
        let knowledge = KnowledgeBase::default();
        let var = variable();

        let action = BuiltinAction::Pure(PureAction::Now(var.clone()));

        let ExecuteResult::Done(Some(result)) = action.execute(&bindings, &mut context, &knowledge)
        else {
            panic!(".now must complete immediately with bindings");
        };

        assert!(matches!(result.get_view(&var), Some(TermView::Number(_))));
    }

    #[test]
    fn test_me_binds_the_agents_fully_qualified_aid_to_the_given_variable() {
        // SAFETY: `.me` never touches the environment.
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let bindings = bindings(vec![]);
        let knowledge = KnowledgeBase::default();
        let var = variable();

        let action = BuiltinAction::Pure(PureAction::Me(var.clone()));

        let ExecuteResult::Done(Some(result)) = action.execute(&bindings, &mut context, &knowledge)
        else {
            panic!(".me must complete immediately with bindings");
        };

        assert_eq!(
            result.get_view(&var).map(TermView::to_owned),
            Some(string("test-agent@local"))
        );
    }

    #[test]
    fn test_append_binds_a_new_list_with_the_item_appended() {
        // SAFETY: `.append` never touches the environment.
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let bindings = bindings(vec![]);
        let knowledge = KnowledgeBase::default();
        let var = variable();

        let action = BuiltinAction::Pure(PureAction::Append(
            list(vec![string("a"), string("b")]),
            string("c"),
            var.clone(),
        ));

        let ExecuteResult::Done(Some(result)) = action.execute(&bindings, &mut context, &knowledge)
        else {
            panic!(".append must complete immediately with bindings");
        };

        assert_eq!(
            result.get_view(&var).map(TermView::to_owned),
            Some(list(vec![string("a"), string("b"), string("c")]))
        );
    }

    #[test]
    fn test_append_fails_when_the_first_argument_is_not_a_list() {
        // SAFETY: `.append` never touches the environment.
        let mut context: Context<()> = unsafe { new_context_without_environment() };
        let bindings = bindings(vec![]);
        let knowledge = KnowledgeBase::default();
        let var = variable();

        let action = BuiltinAction::Pure(PureAction::Append(
            string("not-a-list"),
            string("c"),
            var.clone(),
        ));

        let result = action.execute(&bindings, &mut context, &knowledge);

        assert!(matches!(result, ExecuteResult::Done(None)));
    }

    #[test]
    fn test_variable_or_receiver_resolves_bound_variable_to_receiver() {
        let var = variable();
        let addr = string("receiver-agent@local");
        let bindings = bindings(vec![(var.clone(), TermView::from(&addr))]);

        let resolved = VariableOrReceiver::Variable(var)
            .resolve(&bindings)
            .expect("should resolve");

        assert_eq!(
            resolved,
            VariableOrReceiver::Receiver(Receiver::Single(Aid::local("receiver-agent")))
        );
    }

    #[test]
    fn test_variable_or_receiver_leaves_unbound_variable_unresolved() {
        let var = variable();
        let bindings = bindings(vec![]);

        let resolved = VariableOrReceiver::Variable(var.clone())
            .resolve(&bindings)
            .expect("should resolve");

        assert_eq!(resolved, VariableOrReceiver::Variable(var));
    }

    #[test]
    fn test_variable_or_receiver_fails_when_bound_value_is_not_an_aid() {
        let var = variable();
        let not_an_aid = string("not-an-aid");
        let bindings = bindings(vec![(var.clone(), TermView::from(&not_an_aid))]);

        let err = VariableOrReceiver::Variable(var)
            .resolve(&bindings)
            .unwrap_err();

        assert!(matches!(
            err,
            ResolveFailure::ConversionFailed(FromTermError::IncorrectConversion(
                ConversionError::InvalidAid(_)
            ))
        ));
    }

    #[test]
    fn test_variable_or_receiver_resolves_receiver_unchanged() {
        let receiver = Receiver::Single(Aid::local("receiver-agent"));
        let bindings = bindings(vec![]);

        let resolved = VariableOrReceiver::Receiver(receiver.clone())
            .resolve(&bindings)
            .expect("should resolve");

        assert_eq!(resolved, VariableOrReceiver::Receiver(receiver));
    }

    mod send_literal {
        use alloc::vec;

        use ember_core::message::{Payload, TransportMessage};

        use crate::testing::literal;

        use super::*;

        fn sent_message<'ctx>(context: &'ctx Context<'ctx, ()>) -> &'ctx Message {
            let [
                (
                    TransportMessage {
                        payload: Payload::AclMessage(message),
                        ..
                    },
                    _,
                ),
            ] = context.message_outbox.as_slice()
            else {
                panic!("expected exactly one parsed acl message in the outbox");
            };
            message
        }

        #[test]
        fn resolves_a_bound_variable_receiver_and_pushes_an_inform_message() {
            let mut context: Context<()> = new_context_with_environment();
            let knowledge = KnowledgeBase::default();
            let receiver_var = variable();
            let addr = string("receiver-agent@local");
            let bindings = bindings(vec![(receiver_var.clone(), addr.as_view())]);

            let action = BuiltinAction::Impure(ImpureAction::SendLiteral(
                VariableOrReceiver::Variable(receiver_var),
                Trigger::Addition,
                VariableOrLiteral::Literal(literal("ack", vec![])),
                Box::new([]),
            ));

            let result = action.execute(&bindings, &mut context, &knowledge);

            assert!(matches!(result, ExecuteResult::Done(None)));
            let message = sent_message(&context);
            assert_eq!(message.performative, Performative::Inform);
            assert_eq!(
                message.receiver,
                Some(Receiver::Single(Aid::local("receiver-agent")))
            );
        }

        #[test]
        fn deletion_trigger_maps_to_the_not_understood_performative() {
            let mut context: Context<()> = new_context_with_environment();
            let knowledge = KnowledgeBase::default();
            let bindings = bindings(vec![]);

            let action = BuiltinAction::Impure(ImpureAction::SendLiteral(
                VariableOrReceiver::Receiver(Receiver::Single(Aid::local("receiver-agent"))),
                Trigger::Deletion,
                VariableOrLiteral::Literal(literal("ack", vec![])),
                Box::new([]),
            ));

            action.execute(&bindings, &mut context, &knowledge);

            assert_eq!(
                sent_message(&context).performative,
                Performative::NotUnderstood
            );
        }

        #[test]
        fn resolves_the_literals_arguments_against_the_calling_frames_bindings() {
            let mut context: Context<()> = new_context_with_environment();
            let knowledge = KnowledgeBase::default();
            let var = variable();
            let room_value = string("kitchen");
            let bindings = bindings(vec![(var.clone(), room_value.as_view())]);

            let action = BuiltinAction::Impure(ImpureAction::SendLiteral(
                VariableOrReceiver::Receiver(Receiver::Single(Aid::local("receiver-agent"))),
                Trigger::Addition,
                VariableOrLiteral::Literal(literal("location", vec![variable_term(&var)])),
                Box::new([]),
            ));

            action.execute(&bindings, &mut context, &knowledge);

            let Some(Content::Bdil(BdilContent::Literal(sent))) = &sent_message(&context).content
            else {
                panic!("expected a bdil literal content");
            };
            assert_eq!(sent.functor.0, "location");
        }

        #[test]
        fn leaves_the_outbox_empty_when_the_receiver_fails_to_resolve() {
            let mut context: Context<()> = new_context_with_environment();
            let knowledge = KnowledgeBase::default();
            let bindings = bindings(vec![]);

            let action = BuiltinAction::Impure(ImpureAction::SendLiteral(
                VariableOrReceiver::Variable(variable()),
                Trigger::Addition,
                VariableOrLiteral::Literal(literal("ack", vec![])),
                Box::new([]),
            ));

            let result = action.execute(&bindings, &mut context, &knowledge);

            assert!(matches!(result, ExecuteResult::Done(None)));
            assert!(context.message_outbox.is_empty());
        }

        #[test]
        fn callback_fires_a_request_message_carrying_the_resolved_goal() {
            let mut context: Context<()> = new_context_with_environment();
            let knowledge = KnowledgeBase::default();
            let var = variable();
            let target_value = string("n1");
            let bindings = bindings(vec![(var.clone(), target_value.as_view())]);

            let action = BuiltinAction::Impure(ImpureAction::SendLiteral(
                VariableOrReceiver::Receiver(Receiver::Single(Aid::local("receiver-agent"))),
                Trigger::Addition,
                VariableOrLiteral::Literal(literal("ack", vec![])),
                Box::new([(
                    CallbackKind::OnFailure,
                    VariableOrLiteral::Literal(literal("retry", vec![variable_term(&var)])),
                )]),
            ));

            action.execute(&bindings, &mut context, &knowledge);

            let [(_, callbacks)] = context.message_outbox.as_mut_slice() else {
                panic!("expected exactly one outbox entry");
            };
            assert!(callbacks.on_success.is_none());
            assert!(callbacks.on_retry.is_none());
            assert!(callbacks.on_complete.is_none());
            let on_failure = callbacks
                .on_failure
                .take()
                .expect("on_failure callback should be attached");

            on_failure(&mut context);

            let [_, (fired, _)] = context.message_outbox.as_slice() else {
                panic!("expected the callback to push a second outbox entry");
            };
            let TransportMessage {
                payload: Payload::AclMessage(message),
                ..
            } = fired
            else {
                panic!("expected a parsed acl message");
            };
            assert_eq!(message.performative, Performative::Request);
            assert_eq!(
                message.receiver,
                Some(Receiver::Single(Aid::local("test-agent")))
            );
            let Some(Content::Bdil(BdilContent::Literal(goal))) = &message.content else {
                panic!("expected a bdil literal content");
            };
            let goal = Literal::from(goal.clone());
            assert_eq!(goal.structure.functor.0, "retry");
            assert_eq!(
                goal.structure.arguments.as_deref(),
                Some([string("n1")].as_slice())
            );
        }
    }

    mod at {
        use crate::event::EventSource;
        use crate::plan::GoalKind;
        use crate::testing::literal;

        use super::*;

        #[test]
        fn first_poll_completes_immediately_and_requeues_non_blocking() {
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let knowledge = KnowledgeBase::default();
            let bindings = bindings(vec![]);

            let action = BuiltinAction::at(
                core::time::Duration::from_secs(3600),
                literal("check_again", vec![]),
            );

            let result = action.execute(&bindings, &mut context, &knowledge);

            assert!(matches!(result, ExecuteResult::Pending(_)));
            assert!(context.actions.is_empty());
        }

        #[test]
        fn requeued_copy_stays_pending_until_delay_elapses() {
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let knowledge = KnowledgeBase::default();
            let bindings = bindings(vec![]);

            let action = BuiltinAction::at(
                core::time::Duration::from_secs(3600),
                literal("check_again", vec![]),
            );
            let ExecuteResult::Pending(requeued) =
                action.execute(&bindings, &mut context, &knowledge)
            else {
                unreachable!()
            };
            let result = requeued.execute(&bindings, &mut context, &knowledge);

            assert!(matches!(result, ExecuteResult::Pending(_)));
            assert!(context.events.is_empty());
        }

        #[test]
        fn fires_exactly_one_achieve_event_once_delay_elapses() {
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let knowledge = KnowledgeBase::default();
            let bindings = bindings(vec![]);

            let action =
                BuiltinAction::at(core::time::Duration::ZERO, literal("check_again", vec![]));
            let ExecuteResult::Pending(requeued) =
                action.execute(&bindings, &mut context, &knowledge)
            else {
                unreachable!()
            };
            let result = requeued.execute(&bindings, &mut context, &knowledge);

            assert!(matches!(result, ExecuteResult::Done(None)));
            assert_eq!(context.events.len(), 1);
            let (source, event) = &context.events[0];
            assert!(matches!(source, EventSource::External));
            assert_eq!(event.trigger, Trigger::Addition);
            assert_eq!(event.goal, Some(GoalKind::Achieve));
            assert_eq!(event.event, literal("check_again", vec![]));
        }

        #[test]
        fn resolves_the_goal_against_the_calling_frames_bindings() {
            use crate::testing::{string, variable};

            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let knowledge = KnowledgeBase::default();
            let var = variable();
            let room_value = string("kitchen");
            let bindings = bindings(vec![(var.clone(), room_value.as_view())]);

            let action = BuiltinAction::at(
                core::time::Duration::ZERO,
                literal("go_to", vec![crate::testing::variable_term(&var)]),
            );
            let ExecuteResult::Pending(requeued) =
                action.execute(&bindings, &mut context, &knowledge)
            else {
                unreachable!()
            };
            requeued.execute(&bindings, &mut context, &knowledge);

            let (_, event) = &context.events[0];
            assert_eq!(event.event, literal("go_to", vec![string("kitchen")]));
        }
    }

    mod forall {
        use alloc::vec;

        use crate::event::EventSource;
        use crate::plan::GoalKind;
        use crate::testing::{assert_belief, literal, literal_formula, variable_term};

        use super::*;

        #[test]
        fn no_solutions_emits_no_events_and_completes_immediately() {
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let knowledge = KnowledgeBase::default();
            let bindings = bindings(vec![]);

            let x = variable();
            let action = BuiltinAction::Impure(ImpureAction::Forall {
                query: literal_formula("item", vec![variable_term(&x)]),
                goal: VariableOrLiteral::Literal(literal("process", vec![variable_term(&x)])),
            });

            let result = action.execute(&bindings, &mut context, &knowledge);

            assert!(
                matches!(result, ExecuteResult::Done(None)),
                "forall never stays pending"
            );
            assert!(
                context.events.is_empty(),
                "no solutions means no goals are spawned"
            );
        }

        #[test]
        fn spawns_one_fire_and_forget_achieve_event_per_solution() {
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let mut knowledge = KnowledgeBase::default();
            assert_belief(&mut knowledge, "item", vec![string("a")]);
            assert_belief(&mut knowledge, "item", vec![string("b")]);
            assert_belief(&mut knowledge, "item", vec![string("c")]);
            let bindings = bindings(vec![]);

            let x = variable();
            let action = BuiltinAction::Impure(ImpureAction::Forall {
                query: literal_formula("item", vec![variable_term(&x)]),
                goal: VariableOrLiteral::Literal(literal("process", vec![variable_term(&x)])),
            });

            let result = action.execute(&bindings, &mut context, &knowledge);
            assert!(matches!(result, ExecuteResult::Done(None)));
            assert_eq!(context.events.len(), 3, "one goal event per solution");

            let mut goals: Vec<_> = context
                .events
                .iter()
                .map(|(source, event)| {
                    assert!(
                        matches!(source, EventSource::External),
                        "each solution is spawned as its own independent intention, not tied \
                         to whatever intention ran .forall"
                    );
                    assert_eq!(event.trigger, Trigger::Addition);
                    assert_eq!(event.goal, Some(GoalKind::Achieve));
                    event.event.clone()
                })
                .collect();
            goals.sort();

            let mut expected = vec![
                literal("process", vec![string("a")]),
                literal("process", vec![string("b")]),
                literal("process", vec![string("c")]),
            ];
            expected.sort();

            assert_eq!(goals, expected);
        }

        #[test]
        fn each_solution_keeps_its_own_variables_without_mixing_with_others() {
            // Correlated pairs: every spawned goal must keep X and Y from the *same*
            // underlying belief, never X from one solution paired with Y from another.
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let mut knowledge = KnowledgeBase::default();
            assert_belief(&mut knowledge, "pair", vec![string("a"), string("1")]);
            assert_belief(&mut knowledge, "pair", vec![string("b"), string("2")]);
            let bindings = bindings(vec![]);

            let (x, y) = (variable(), variable());
            let action = BuiltinAction::Impure(ImpureAction::Forall {
                query: literal_formula("pair", vec![variable_term(&x), variable_term(&y)]),
                goal: VariableOrLiteral::Literal(literal(
                    "process",
                    vec![variable_term(&x), variable_term(&y)],
                )),
            });

            let result = action.execute(&bindings, &mut context, &knowledge);
            assert!(matches!(result, ExecuteResult::Done(None)));

            let mut goals: Vec<_> = context.events.into_iter().map(|(_, e)| e.event).collect();
            goals.sort();

            let mut expected = vec![
                literal("process", vec![string("a"), string("1")]),
                literal("process", vec![string("b"), string("2")]),
            ];
            expected.sort();

            assert_eq!(
                goals, expected,
                "no solution's goal may combine X from one pair with Y from another"
            );
        }

        #[test]
        fn goal_sees_both_the_calling_frames_bindings_and_the_querys_bindings() {
            let mut context: Context<()> = unsafe { new_context_without_environment() };
            let mut knowledge = KnowledgeBase::default();
            assert_belief(&mut knowledge, "item", vec![string("a")]);
            assert_belief(&mut knowledge, "item", vec![string("b")]);

            let (x, room) = (variable(), variable());
            let room_value = string("kitchen");
            let bindings = bindings(vec![(room.clone(), room_value.as_view())]);

            let action = BuiltinAction::Impure(ImpureAction::Forall {
                query: literal_formula("item", vec![variable_term(&x)]),
                goal: VariableOrLiteral::Literal(literal(
                    "process",
                    vec![variable_term(&x), variable_term(&room)],
                )),
            });

            let result = action.execute(&bindings, &mut context, &knowledge);
            assert!(matches!(result, ExecuteResult::Done(None)));

            let mut goals: Vec<_> = context.events.into_iter().map(|(_, e)| e.event).collect();
            goals.sort();

            let mut expected = vec![
                literal("process", vec![string("a"), string("kitchen")]),
                literal("process", vec![string("b"), string("kitchen")]),
            ];
            expected.sort();

            assert_eq!(goals, expected);
        }
    }
}
