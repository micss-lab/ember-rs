use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

use crate::ast::{Belief, Goal, Plan, Program};
use crate::compiler::AstVisitor;

pub(crate) fn expand(asl: &Program, agent_name: &str, agent_ident: &Ident) -> TokenStream {
    let Program {
        beliefs,
        goals,
        plans,
    } = asl;

    let beliefbase = generate_beliefbase(beliefs, agent_ident);
    let initial_goals = generate_initial_goals(goals, agent_ident);
    let plan_library = generate_plan_library(plans, agent_ident);

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

fn generate_beliefbase(beliefs: &[Belief], agent_ident: &Ident) -> impl ToTokens {
    let beliefs = beliefs.into_iter().map(|b| {
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let belief = visitor.visit_belief(b).into_token_stream();
        let variables = visitor
            .variable_map
            .into_values()
            .map(|v| {
                quote! {
                    let #v = ::ember::agent::bdi::variable::Variable::new();
                }
            })
            .collect::<Vec<_>>();

        quote! { {
            #(#variables)*
            #belief
        } }
    });

    quote! {
        ::ember::agent::bdi::knowledge::store::BeliefBase::from_iter([
            #(#beliefs),*
        ])
    }
}

fn generate_initial_goals(goals: &[Goal], agent_ident: &Ident) -> impl ToTokens {
    let goals = goals.into_iter().map(|g| {
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let goal = visitor.visit_goal(g).into_token_stream();
        let variables = visitor
            .variable_map
            .into_values()
            .map(|v| {
                quote! {
                    let #v = ::ember::agent::bdi::variable::Variable::new();
                }
            })
            .collect::<Vec<_>>();

        quote! { {
            #(#variables)*
            #goal
        } }
    });

    quote! {
        ::alloc::vec::Vec::from([
            #(#goals),*
        ])
    }
}

fn generate_plan_library(plans: &[Plan], agent_ident: &Ident) -> impl ToTokens {
    let plans = plans.into_iter().map(|p| {
        let mut visitor = AstVisitor::new(agent_ident.clone());
        let plan = visitor.visit_plan(p).into_token_stream();
        let variables = visitor.variable_map.into_values().map(|v| {
            quote! {
                let #v = ::ember::agent::bdi::variable::Variable::new();
            }
        });

        quote! { {
            #(#variables)*
            #plan
        } }
    });

    quote! { {
        let mut plans = ::ember::agent::bdi::plan::library::PlanLibrary::default();
        #(
            plans.add(#plans);
        )*
        plans
    } }
}
