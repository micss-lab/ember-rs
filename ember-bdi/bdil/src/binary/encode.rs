use alloc::vec::Vec;

use crate::error::EncodeError;
use crate::{BdilContent, Functor, Literal, Term, Variable};

use super::codec::*;

pub fn encode(content: &BdilContent) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.push(VER_0_1_0);
    let result = match content {
        BdilContent::Literal(l) => push_expression(l, &mut out)
            .map_err(|e| log::error!("failed to encode literal bdil content: {e}")),
    };
    if let Err(_) = result {
        return Vec::with_capacity(0);
    }
    out.push(END);
    out
}

fn push_expression(literal: &Literal, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    out.push(if literal.negated {
        EXPR_LIT_NEG
    } else {
        EXPR_LIT_POS
    });
    push_literal_body(literal, out)
}

fn push_literal_body(literal: &Literal, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    push_functor(&literal.functor, out)?;
    push_arg_list(literal.arguments.as_deref(), out)
}

fn push_functor(functor: &Functor, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    let bytes = functor.0.as_bytes();
    if bytes.is_empty() {
        return Err(EncodeError::EmptyFunctor);
    }
    if bytes.contains(&0x00) {
        return Err(EncodeError::FunctorContainsNull);
    }
    out.push(WORD);
    out.extend_from_slice(bytes);
    out.push(0x00);
    Ok(())
}

fn push_arg_list(args: Option<&[Term]>, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    for term in args.unwrap_or(&[]) {
        push_term(term, out)?;
    }
    out.push(END);
    Ok(())
}

fn push_term(term: &Term, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    match term {
        Term::Int(i) => {
            out.push(T_INT);
            out.extend_from_slice(&i.to_le_bytes());
        }
        Term::Float(f) => {
            out.push(T_FLT);
            out.extend_from_slice(&f.0.to_le_bytes());
        }
        Term::String(s) => {
            out.push(T_STR);
            out.extend_from_slice(&(s.len() as u16).to_be_bytes());
            out.extend_from_slice(s.as_slice());
        }
        Term::Literal(lit) => {
            out.push(if lit.negated { T_LIT_NEG } else { T_LIT_POS });
            push_literal_body(lit, out)?;
        }
        Term::Variable(var) => push_variable(var, out)?,
    }
    Ok(())
}

fn push_variable(var: &Variable, out: &mut Vec<u8>) -> Result<(), EncodeError> {
    let bytes = var.name.as_bytes();
    if bytes.is_empty() {
        return Err(EncodeError::VariableNameEmpty);
    }
    if bytes.contains(&0x00) {
        return Err(EncodeError::VariableNameContainsNull);
    }
    out.push(T_VAR);
    out.extend_from_slice(bytes);
    out.push(0x00);
    Ok(())
}
