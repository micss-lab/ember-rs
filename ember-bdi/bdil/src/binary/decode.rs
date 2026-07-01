use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;

use bstr::BString;
use ember_util::cmp::TotalCmpF32;

use crate::error::DecodeError;
use crate::{Functor, Literal, Term, Variable};

use super::codec::*;
use super::parsing::Bytes;

pub fn decode(data: &[u8]) -> Result<Literal, DecodeError> {
    parser::frame(&Bytes::from(data)).map_err(|_| DecodeError::ParseFailed)
}

peg::parser! {
    grammar parser<'a>() for Bytes<'a> {

        pub rule frame() -> Literal
            = magic() version() e:expr() [END] ![_] { e }

        rule magic()
            = [0xCA] [0xED]

        rule version()
            = [VER_0_1_0]
            / [VER_EXPLICIT] major:[_] [_] [_]
              {? if major == 0 { Ok(()) } else { Err("unsupported version") } }

        rule expr() -> Literal
            = [EXPR_LIT_POS] body:literal_body()
              { let (f, a) = body; Literal { negated: false, functor: f, arguments: a } }
            / [EXPR_LIT_NEG] body:literal_body()
              { let (f, a) = body; Literal { negated: true, functor: f, arguments: a } }

        rule literal_body() -> (Functor, Option<Box<[Term]>>)
            = f:functor() args:arg_list() { (f, args) }

        rule functor() -> Functor
            = [WORD] bytes:$([b if b != 0x00]+) [0x00]
              {? String::from_utf8(bytes.to_vec()).map(Functor).map_err(|_| "invalid utf-8") }

        rule arg_list() -> Option<Box<[Term]>>
            = terms:term()* [END]
              { if terms.is_empty() { None } else { Some(terms.into_boxed_slice()) } }

        rule term() -> Term
            = [T_INT] b:$([_]*<4>)
              { Term::Int(i32::from_le_bytes([b[0], b[1], b[2], b[3]])) }
            / [T_FLT] b:$([_]*<4>)
              { Term::Float(TotalCmpF32(f32::from_le_bytes([b[0], b[1], b[2], b[3]]))) }
            / [T_STR] hi:[_] lo:[_] bytes:$([_]*<{(hi as usize) << 8 | lo as usize}>)
              { Term::Str(BString::from(bytes)) }
            / [T_LIT_POS] body:literal_body()
              { let (f, a) = body; Term::Literal(Literal { negated: false, functor: f, arguments: a }) }
            / [T_LIT_NEG] body:literal_body()
              { let (f, a) = body; Term::Literal(Literal { negated: true, functor: f, arguments: a }) }
            / [T_VAR] bytes:$([b if b != 0x00]+) [0x00]
              {? String::from_utf8(bytes.to_vec())
                     .map(|name| Term::Variable(Variable { name }))
                     .map_err(|_| "invalid utf-8") }
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    extern crate std;

    use alloc::boxed::Box;

    use crate::binary::encode::encode;
    use crate::{Functor, Literal, Term, Variable};

    use super::decode;

    fn round_trip(lit: Literal) {
        let bytes = encode(&lit).expect("encode failed");
        let decoded = decode(&bytes).expect("decode failed");
        assert_eq!(lit, decoded);
    }

    #[test]
    fn atom() {
        round_trip(Literal { negated: false, functor: Functor("raining".into()), arguments: None });
    }

    #[test]
    fn negated_atom() {
        round_trip(Literal { negated: true, functor: Functor("sunny".into()), arguments: None });
    }

    #[test]
    fn literal_with_literal_args() {
        round_trip(Literal {
            negated: false,
            functor: Functor("location".into()),
            arguments: Some(Box::new([
                Term::Literal(Literal {
                    negated: false,
                    functor: Functor("agent1".into()),
                    arguments: None,
                }),
                Term::Literal(Literal {
                    negated: false,
                    functor: Functor("room3".into()),
                    arguments: None,
                }),
            ])),
        });
    }

    #[test]
    fn literal_with_int_and_float() {
        round_trip(Literal {
            negated: false,
            functor: Functor("reading".into()),
            arguments: Some(Box::new([Term::Int(42), Term::Float(3.14f32.into())])),
        });
    }

    #[test]
    fn literal_with_string() {
        round_trip(Literal {
            negated: false,
            functor: Functor("label".into()),
            arguments: Some(Box::new([Term::Str(b"hello world".into())])),
        });
    }

    #[test]
    fn literal_with_variable() {
        round_trip(Literal {
            negated: false,
            functor: Functor("at".into()),
            arguments: Some(Box::new([
                Term::Literal(Literal {
                    negated: false,
                    functor: Functor("robot".into()),
                    arguments: None,
                }),
                Term::Variable(Variable { name: "X".into() }),
            ])),
        });
    }

    #[test]
    fn negated_nested_literal() {
        round_trip(Literal {
            negated: false,
            functor: Functor("state".into()),
            arguments: Some(Box::new([Term::Literal(Literal {
                negated: true,
                functor: Functor("broken".into()),
                arguments: None,
            })])),
        });
    }

    #[test]
    fn invalid_magic() {
        assert!(decode(&[0xFF, 0xFF, 0x40, 0x10, 0x30, b'a', 0x00, 0x00, 0x00]).is_err());
    }

    #[test]
    fn empty_frame_is_missing_expression() {
        assert!(decode(&[0xCA, 0xED, 0x40, 0x00]).is_err());
    }
}
