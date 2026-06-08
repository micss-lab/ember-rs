use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

use crate::ast::*;

pub(crate) fn compile_asl(asl: &Program, agent_name: &str, agent_ident: &Ident) -> TokenStream {
    let Program {
        beliefs,
        goals,
        plans,
    } = asl;

    let beliefbase = generate_beliefbase(beliefs);

    quote! {
        impl From<#agent_ident> for ::ember::agent::bdi::BdiAgent<'static, #agent_ident, (), ()>  {
            fn from(agent: #agent_ident) -> Self {
                let beliefbase = #beliefbase;

                todo!()
            }
        }
    }
}

fn generate_beliefbase(beliefs: &[Belief]) -> impl ToTokens {
    let mut visitor = AstVisitor::default();
    let beliefs = beliefs
        .into_iter()
        .map(|b| visitor.visit_belief(b).into_token_stream())
        .collect::<Vec<_>>();
    let variables = visitor
        .variable_map
        .into_values()
        .map(|v| quote! { let #v = ::ember::agent::bdi::variable::Variable::new(); })
        .collect::<Vec<_>>();

    quote! {
        {
            #(#variables)*
            ::ember::agent::bdi::knowledge::store::BeliefBase::from_iter([
                #(#beliefs),*
            ])
        }
    }
}

#[derive(Default)]
struct AstVisitor {
    /// Maps from a `Variable` name to the ident name of the generated rust variable.
    variable_map: HashMap<String, Ident>,
}

impl AstVisitor {
    fn visit_belief(&mut self, Belief(literal): &Belief) -> impl ToTokens {
        let literal = self.visit_literal(literal);
        quote! {
            ::ember::agent::bdi::knowledge::belief::Belief::from(
                #literal
            )
        }
    }

    fn visit_literal(&mut self, Literal { negated, formula }: &Literal) -> impl ToTokens {
        let structure = self.visit_atomic_formula(formula);
        quote! {
            ::ember::agent::bdi::literal::Literal::Atom {
                negated: #negated,
                structure: #structure
            }
        }
    }

    fn visit_atomic_formula(
        &mut self,
        AtomicFormula { functor, arguments }: &AtomicFormula,
    ) -> impl ToTokens {
        let arguments = match arguments {
            Some(args) => {
                let args = args
                    .into_iter()
                    .map(|a| self.visit_term(a).into_token_stream())
                    .collect::<Vec<_>>();
                quote! {
                    ::core::option::Option::Some(::alloc::boxed::Box::new([
                        #(#args),*
                    ]))
                }
            }
            None => quote! { ::core::option::Option::None },
        };
        let functor = self.visit_atom_or_var(functor);
        quote! {
            ::ember::agent::bdi::term::Structure {
                functor: #functor,
                arguments: #arguments,
            }
        }
    }

    fn visit_term(&mut self, term: &Term) -> impl ToTokens {
        match term {
            Term::Literal(Literal { negated, formula }) => {
                let formula = self.visit_atomic_formula(formula);
                quote! {
                    ::ember::agent::bdi::term::owned::Term::Literal {
                        negated: #negated,
                        structure: #formula,
                    }
                }
            }
            Term::Variable(variable) => {
                let variable = self.visit_variable(variable);
                quote! {
                    ::ember::agent::bdi::term::owned::Term::Variable(#variable)
                }
            }
            Term::Number(number) => quote! {
                ::ember::agent::bdi::term::owned::Term::Number(#number.into()),
            },
            Term::String(string) => {
                quote! {
                    ::ember::agent::bdi::term::owned::Term::String(#string)
                }
            }
        }
    }

    fn visit_variable(&mut self, Variable(name): &Variable) -> impl ToTokens {
        let var_name = self
            .variable_map
            .entry(name.clone())
            .or_insert_with(|| format_ident!("var_{}", name.to_lowercase()));
        quote! {
            #var_name
        }
    }

    fn visit_atom_or_var(&mut self, atom_or_var: &AtomOrVar) -> impl ToTokens {
        match atom_or_var {
            AtomOrVar::Variable(_) => {
                quote! { compile_error!("variables as functors are not (yet) supported") }
            }
            AtomOrVar::Atom(atom) => self.visit_atom(atom).into_token_stream(),
        }
    }

    fn visit_atom(&mut self, Atom(value): &Atom) -> impl ToTokens {
        quote! {
            ::ember::agent::bdi::term::owned::Atom(#value.into())
        }
    }
}
