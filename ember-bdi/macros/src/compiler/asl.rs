use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

use crate::ast::{Belief, Goal, Plan, Program, Spanned};
use crate::compiler::AstVisitor;

pub(crate) fn expand(asl: &Program, agent_ident: &Ident) -> TokenStream {
    use heck::ToKebabCase;

    let Program {
        beliefs,
        goals,
        plans,
    } = asl;

    let beliefbase = generate_beliefbase(beliefs, agent_ident);
    let initial_goals = generate_initial_goals(goals, agent_ident);
    let plan_library = generate_plan_library(plans, agent_ident);

    let agent_name = agent_ident.to_string().to_kebab_case();
    let agent_action = format_ident!("{}Action", agent_ident);

    quote! {
        impl From<#agent_ident> for ::ember::agent::bdi::BdiAgent<'static, #agent_ident, #agent_action, ()> {
            fn from(agent: #agent_ident) -> Self {
                let beliefbase = #beliefbase;
                let initial_goals = #initial_goals;
                let plan_library = #plan_library;

                ::ember::agent::bdi::BdiAgent::new(
                    #agent_name,
                    agent,
                    [],
                    Some(beliefbase),
                    plan_library,
                    initial_goals,
                )
            }
        }

        impl #agent_ident {
            fn into_agent(self) -> ::ember::agent::bdi::BdiAgent<'static, #agent_ident, #agent_action, ()> {
                self.into()
            }
        }
    }
}

fn generate_beliefbase(beliefs: &[Spanned<Belief>], agent_ident: &Ident) -> impl ToTokens {
    let beliefs = beliefs.into_iter().map(|b| {
        let span = b.span;
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let belief = visitor.visit_belief(&b.node).into_token_stream();
        let variables = visitor
            .variable_map
            .into_values()
            .map(|v| {
                quote! {
                    let #v = ::ember::agent::bdi::variable::Variable::new();
                }
            })
            .collect::<Vec<_>>();

        quote::quote_spanned! { span=>
            let _belief = {
                #(#variables)*
                #belief
            };
            ::ember::agent::bdi::knowledge::store::BeliefBase::assert_no_event(&mut beliefbase, _belief);
        }
    });

    quote! { {
        let mut beliefbase = ::ember::agent::bdi::knowledge::store::BeliefBase::default();
        #(#beliefs)*
        beliefbase
    } }
}

fn generate_initial_goals(goals: &[Spanned<Goal>], agent_ident: &Ident) -> impl ToTokens {
    let goals = goals.into_iter().map(|g| {
        let span = g.span;
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let goal = visitor.visit_goal(&g.node).into_token_stream();
        let variables = visitor
            .variable_map
            .into_values()
            .map(|v| {
                quote! {
                    let #v = ::ember::agent::bdi::variable::Variable::new();
                }
            })
            .collect::<Vec<_>>();

        quote::quote_spanned! { span=>
            let _goal = {
                #(#variables)*
                #goal
            };
            goals.push(_goal);
        }
    });

    quote! { {
        let mut goals = ::alloc::vec::Vec::new();
        #(#goals)*
        goals
    } }
}

fn generate_plan_library(plans: &[Spanned<Plan>], agent_ident: &Ident) -> impl ToTokens {
    let plans = plans.into_iter().map(|p| {
        let span = p.span;
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let plan = visitor.visit_plan(&p.node).into_token_stream();
        let variables = visitor.variable_map.into_values().map(|v| {
            quote! {
                let #v = ::ember::agent::bdi::variable::Variable::new();
            }
        });

        quote::quote_spanned! { span=>
            let _plan = {
                #(#variables)*
                #plan
            };
            plans.add(_plan);
        }
    });

    quote! { {
        let mut plans = ::ember::agent::bdi::plan::library::PlanLibrary::default();
        #(#plans)*
        plans
    } }
}
