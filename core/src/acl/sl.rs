use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use bstr::ByteSlice;
use core::fmt::Write;
use core::marker::PhantomData;
use core::ops::Deref;

use chrono::{DateTime, FixedOffset};

/// List of expressions to form the content of an ACL message.
#[derive(Debug, Clone, PartialEq)]
pub struct Content(pub Vec<ContentElement>);

impl Content {
    pub fn parse(input: impl AsRef<bstr::BStr>) -> Result<Self, String> {
        parser::sl0_content::content(&parser::BStr::from(input.as_ref())).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentElement {
    AgentAction(AgentAction),
    Predicate(Predicate),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Concept {
    /// Type defining the concept.
    pub symbol: bstr::BString,
    /// Parameters belonging to the concept.
    pub parameters: ConceptParameters,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentAction {
    /// Agent performing the action.
    pub agent: Term,
    /// The action to be performed.
    pub action: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    Regular {
        symbol: bstr::BString,
        terms: Vec<Term>,
    },
    Result {
        lhs: Term,
        rhs: Term,
    },
    Done {
        action: AgentAction,
    },
    Bool(bool),
}

/// Recursive structure defining the concept of a term.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Constant(Constant),
    Set(Set),
    Sequence(Seq),
    Concept(Concept),
    Action(Box<AgentAction>),
}

/// Parameters part of a functional term.
#[derive(Debug, Clone, PartialEq)]
pub enum ConceptParameters {
    Positional(Vec<Term>),
    ByName(Vec<(bstr::BString, Term)>),
}

/// Numerical, string, and time-related constants.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Number(Number),
    String(bstr::BString),
    Datatime(DateTime<FixedOffset>),
}

/// Numerical constant.
#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int(i32),
    Float(f32),
}

pub type Set = Collection<collection::SetLike>;
pub type Seq = Collection<collection::SeqLike>;

/// General collection type.
///
/// Note: A set cannot be stored in a [`BTreeSet`] as it requires the
/// items to be [`Ord`]. They cannot be as this would require evaluating
/// the terms before storing them.
///
/// [`BTreeSet`]: alloc::collections::btree_set::BTreeSet
/// [`Ord`]: core::cmp::Ord
#[derive(Debug, Clone, PartialEq)]
pub struct Collection<C> {
    /// Items in the collection.
    items: Vec<Term>,
    /// Semantics behind the collection.
    _marker: PhantomData<C>,
}
mod collection {
    #[derive(Debug, Clone, PartialEq)]
    pub struct SetLike;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SeqLike;
}

impl<C> Deref for Collection<C> {
    type Target = [Term];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<C> FromIterator<Term> for Collection<C> {
    fn from_iter<T: IntoIterator<Item = Term>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
            _marker: PhantomData,
        }
    }
}

mod parser {
    pub(super) use self::input::BStr;

    mod input {
        pub(crate) struct BStr<'a>(&'a bstr::BStr);

        impl<'a> From<&'a bstr::BStr> for BStr<'a> {
            fn from(value: &'a bstr::BStr) -> Self {
                Self(value)
            }
        }

        impl<'a> core::ops::Deref for BStr<'a> {
            type Target = bstr::BStr;

            fn deref(&self) -> &Self::Target {
                self.0
            }
        }

        pub struct LineCol {
            /// Byte offset from start of string (0-indexed).
            pub offset: usize,

            /// Line (1-indexed).
            pub line: usize,

            /// Column (1-indexed).
            pub column: usize,
        }

        impl core::fmt::Display for LineCol {
            fn fmt(
                &self,
                fmt: &mut ::core::fmt::Formatter,
            ) -> ::core::result::Result<(), ::core::fmt::Error> {
                write!(
                    fmt,
                    "({}:{})[byte: {}]",
                    self.line, self.column, self.offset
                )
            }
        }

        impl<'a> peg::Parse for BStr<'a> {
            type PositionRepr = LineCol;

            #[inline]
            fn start(&self) -> usize {
                0
            }

            #[inline]
            fn is_eof(&self, pos: usize) -> bool {
                pos >= self.len()
            }

            fn position_repr(&self, pos: usize) -> LineCol {
                use bstr::ByteSlice;

                let before = &self[..pos];
                let line = before.iter().filter(|&&c| c == b'\n').count() + 1;
                let column = before.chars().rev().take_while(|&c| c != '\n').count() + 1;
                LineCol {
                    line,
                    column,
                    offset: pos,
                }
            }
        }

        impl<'a, 'input> peg::ParseElem<'input> for BStr<'a> {
            type Element = u8;

            #[inline]
            fn parse_elem(&'input self, pos: usize) -> peg::RuleResult<u8> {
                match self[pos..].first() {
                    Some(c) => peg::RuleResult::Matched(pos + 1, *c),
                    None => peg::RuleResult::Failed,
                }
            }
        }

        impl<'a> peg::ParseLiteral for BStr<'a> {
            #[inline]
            fn parse_string_literal(&self, pos: usize, literal: &str) -> peg::RuleResult<()> {
                let l = literal.len();
                if self.len() >= pos + l && self[pos..pos + l] == literal.as_bytes() {
                    peg::RuleResult::Matched(pos + l, ())
                } else {
                    peg::RuleResult::Failed
                }
            }
        }
        impl<'a, 'input> peg::ParseSlice<'input> for BStr<'a> {
            type Slice = &'input bstr::BStr;

            #[inline]
            fn parse_slice(&'input self, p1: usize, p2: usize) -> &'input bstr::BStr {
                &self[p1..p2]
            }
        }
    }

    peg::parser! {
        pub(super) grammar sl0_content<'a>() for input::BStr<'a> {
            use bstr::ByteSlice;
            use chrono::{DateTime, FixedOffset, Utc};

            use super::super::*;

            rule _ = [c if c.is_ascii_whitespace()]?

            pub rule content() -> Content
                = lbrace() _ c:(c:content_expression() _ { c })+ _ rbrace() { Content(c) }

            rule content_expression() -> ContentElement
                = a:action_expression() { ContentElement::AgentAction(a) }
                / p:proposition() { ContentElement::Predicate(p) }

            rule proposition() -> Predicate
                = wff()

            rule action_expression() -> AgentAction
                = lbrace() _ "action" _ a:agent() _ t:term() _ rbrace() { AgentAction { agent: a, action: t } }

            rule wff() -> Predicate
                 = atomic_formula()
                / lbrace() _ "done" _ a:action_expression() _ rbrace() { Predicate::Done { action: a } }

            rule atomic_formula() -> Predicate
                = s:propostion_symbol() { Predicate::Regular { symbol: s, terms: Vec::with_capacity(0) }}
                / lbrace() _ "result" _ lhs:term() _ rhs:term() _ rbrace() { Predicate::Result { lhs, rhs } }
                / lbrace() _ s:predicate_symbol() _ t:(t:term() _ { t })+ _ rbrace() { Predicate::Regular { symbol: s, terms: t } }
                / "true" { Predicate::Bool(true) }
                / "false" { Predicate::Bool(false) }

            rule term() -> Term
                = c:constant() { Term::Constant(c) }
                / s:set() { Term::Set(s) }
                / s:sequence() { Term::Sequence(s) }
                / ft:functional_term() { Term::Concept(ft) }
                / ae:action_expression() { Term::Action(Box::new(ae)) }


            rule functional_term() -> Concept
                = lbrace() _ s:function_symbol() _ p:(
                    t:(t:term() _ { t })* { ConceptParameters::Positional(t) }
                    / p:(p:parameter() _ { p })* { ConceptParameters::ByName(p) }
                  ) _ rbrace() { Concept {symbol: s, parameters: p} }

            rule parameter() -> (bstr::BString, Term)
                = n:parameter_name() _ v:parameter_value() { (n, v) }

            rule parameter_value() -> Term
                = term()

            rule agent() -> Term
                = term()

            // ====================
            //      Constants
            // ====================

            rule function_symbol() -> bstr::BString
                = string()

            rule propostion_symbol() -> bstr::BString
                = string()

            rule predicate_symbol() -> bstr::BString
                = string()

            rule constant() -> Constant
                = n:number() { Constant::Number(n) }
                / s:string() { Constant::String(s) }
                / d:datetime() { Constant::Datatime(d) }

            rule parameter_name() -> bstr::BString
                 = colon() _ s:string() { s }

            // ====================
            //     Collections
            // ====================

            // TODO: Collect the items.

            rule sequence() -> Seq
                = lbrace() _ "sequence" t:(t:term() _ { t })* _ rbrace() { Collection { items: t, _marker: PhantomData } }

            rule set() -> Set
                = lbrace() _ "set" t:(t:term() _ { t })* _ rbrace() { Collection { items: t, _marker: PhantomData } }

            // ====================
            //      Numerical
            // ====================

            /// A single decimal character (not parsed).
            rule is_decimal()
                = [c if c.is_ascii_digit()]

            rule is_decimal_with_size(size: u32)
                = is_decimal()*<{size as usize}>

            /// An integer with a specified decimal character length.
            rule uint_with_size(size: u8) -> u32
                = i:$(is_decimal_with_size(size as u32)) {? i.to_str().map_err(|_| "u32")?.parse().map_err(|_| "u32") }

            /// An integer represented by a minimum of one character.
            ///
            /// Note: A bit size of 32 is chosen as the FIPA standard does not specify a hard maximum.
            rule uint() -> u32
                // Decimal.
                = n:$(is_decimal()+) {? n.to_str().map_err(|_| "u32")?.parse().map_err(|_| "u32") }
                // Hexadecimal.
                / n:$([b'0'] [b'x' | b'X'] is_decimal()+) {? n.to_str().map_err(|_| "u32")?.parse().map_err(|_| "u32") }

            /// An integer represented by a minimum of one character and an optional sign.
            ///
            /// Note: A bit size of 32 is chosen as the FIPA standard does not specify a hard maximum.
            rule int() -> i32
                = s:neg_sign() i:uint() {? Ok(i32::try_from(i).map_err(|_| "i32")? * i32::from(s)) }

            /// A floating point value represented by a mantissa and an optional exponent.
            rule float() -> f32
                = s:neg_sign() f:float_mantissa() e:float_exponent()? { f * if let Some(e) = e { libm::powf(10f32, e as f32) } else { 1.0 } * i32::from(s) as f32 }

            /// A floating point value's mantissa.
            rule float_mantissa() -> f32
                = f:$(is_decimal()+ dot()? is_decimal()*) {? f.to_str().map_err(|_| "f32")?.parse().map_err(|_| "f32") }
                / f:$(is_decimal()* dot()? is_decimal()+) {? f.to_str().map_err(|_| "f32")?.parse().map_err(|_| "f32") }

            /// A floating point value's exponent including the 'e' character.
            rule float_exponent() -> i32
                = [b'e' | b'E'] i:int() { i }

            /// Numerical constant.
            rule number() -> Number
                = i:int() { Number::Int(i) }
                / f:float() { Number::Float(f)}

            // ====================
            //       Strings
            // ====================

            /// Regular quoted string literal.
            rule string_literal() -> bstr::BString
                = quote() s:$(("\\\"" / [^b'"'])*) quote() { s.into() }

            /// Single word that is a valid variable name.
            rule word() -> bstr::BString
                = s:$([^(0x00..=0x20 | b'(' | b')' | b'#' | b'0'..=b'9' | b':' | b'-' | b'?')][^(0x00..=0x20 | b'(' | b')')]*) {
                s.into()
            }

            /// Byte string with an encoded byte length.
            rule byte_length_encoded_string() -> bstr::BString
                = hashtag() n:uint() quote() s:$([_]*<{n as usize}>) { s.into() }

            rule string() -> bstr::BString
                = word() / byte_length_encoded_string() / string_literal()

            // ====================
            //       Datetime
            // ====================

            rule datetime() -> DateTime<FixedOffset>
                = s:sign() d:$(year() month() day() time_sep() hour() minute() second() millisecond()) t:opt_timezone() {?
                    // TODO: Handle sign as relative time.

                    let Ok(d) = d.to_str() else {
                        return Err("datetime");
                    };
                    match chrono::NaiveDateTime::parse_from_str(d, "%Y%m%dT%H%M%S%.3f") {
                        Ok(d) => Ok(d.and_local_timezone(t).single().expect("timezone should be non-ambiguous")),
                        Err(_) => Err("datetime"),
                    }
                }

            rule year() = is_decimal_with_size(4)
            rule month() = is_decimal_with_size(2)
            rule day() = is_decimal_with_size(2)
            rule hour() = is_decimal_with_size(2)
            rule minute() = is_decimal_with_size(2)
            rule second() = is_decimal_with_size(2)
            rule millisecond() = is_decimal_with_size(3)

            /// Timezone of the datetime.
            ///
            /// If not specified, utc is used.
            ///
            /// Note: The local time cannot be determined without the standard
            /// library.
            rule opt_timezone() -> FixedOffset
                = c:[b'a'..=b'z' | b'A'..=b'Z']? {?
                    use chrono::Offset;

                    let Some(c) = c else {
                        return Ok(Utc.fix());
                    };

                    Ok(match c.to_ascii_lowercase() {
                        b'z' => Utc.fix(),
                        _ => return Err("timezone"),
                    })
                }

            // ====================
            //       Symbols
            // ====================

            rule dot() = [b'.']

            /// Whether the rule matched a sign.
            ///
            /// Negative is represented as true.
            rule sign() -> Option<bool> = s:[b'-' | b'+']? { s.map(|s| s == b'-') }

            /// Whether the rule matched a sign that is negative.
            rule neg_sign() -> bool = s:sign() { s.is_some_and(|s| s) }

            rule lbrace() = [b'(']
            rule rbrace() = [b')']

            rule semi() = [b';']
            rule colon() = [b':']

            rule vert() = [b'|']

            rule eq() = [b'=']

            rule time_sep() = [b'T']

            rule hashtag() = [b'#']

            rule quote() = [b'"']
        }
    }

    #[cfg(test)]
    mod tests {
        extern crate std;

        use super::super::Content;

        mod parse_success {
            use alloc::string::String;

            use super::Content;

            fn parse(input: &str) -> Result<(), String> {
                Content::parse(input).map(|_| ())
            }

            #[test]
            fn empty_content() {
                parse("()").unwrap_err();
            }

            #[test]
            fn simple_proposition() {
                parse("(some_proposition)").unwrap();
            }

            #[test]
            fn complex_content_expression() {
                parse(
                    "((action agent1 term1) (predicate term1 term2) (done (action agent2 term2)))",
                )
                .unwrap();
            }

            #[test]
            fn atomic_formulas() {
                parse("(true)").unwrap();
                parse("(false)").unwrap();
                parse("((result term1 term2))").unwrap();
                parse("((predicate term1 term2 term3))").unwrap();
            }

            #[test]
            fn action_expressions() {
                parse("((action agent1 term1))").unwrap();
                parse("((action (function param1 value1) term2))").unwrap();
            }

            #[test]
            fn nested_action_expressions() {
                parse("((action agent1 (action agent2 term2)))").unwrap();
            }

            #[test]
            fn done_operator() {
                parse("((done (action agent1 term1)))").unwrap();
            }

            #[test]
            fn functional_terms() {
                parse("((function term1 term2))").unwrap();
                parse("((function param1 value1 param2 value2))").unwrap();
            }

            #[test]
            fn mixed_functional_terms() {
                parse("((function term1 param1 value1 term2))").unwrap();
            }

            #[test]
            fn complex_nested_structure() {
                parse(
                "((action agent1 (set (sequence 1 2 3) (function param1 (action agent2 term2)))))",
            )
            .unwrap();
            }

            #[test]
            fn multiple_content_expressions() {
                parse("(proposition1 proposition2 (action agent1 term1) (done (action agent2 term2)))")
                .unwrap();
            }

            #[test]
            fn complex_predicate() {
                parse(
                "((complex_predicate (function param1 value1) (set 1 2 3) (sequence term1 term2)))",
            )
            .unwrap();
            }

            #[test]
            fn nested_functional_terms() {
                parse("((outer_function (inner_function1 term1) (inner_function2 param1 value1)))")
                    .unwrap();
            }

            #[test]
            fn mixed_term_types() {
                parse("((action agent1 (set 1 \"string\" 3.14 (sequence term1 term2))))").unwrap();
            }

            #[test]
            fn complex_result() {
                parse("((result (function param1 value1) (set 1 2 3)))").unwrap();
            }

            #[test]
            fn nested_done_operators() {
                parse("((done (done (action agent1 term1))))").unwrap();
            }

            #[test]
            fn empty_set_and_sequence() {
                parse("((action agent1 (set) (sequence)))").unwrap();
            }

            #[test]
            fn complex_parameter_values() {
                parse("((function param1 (set 1 2 3) param2 (sequence term1 term2)))").unwrap();
            }

            #[test]
            fn deeply_nested_structure() {
                parse("((action agent1 (function param1 (set (sequence 1 2 3) (action agent2 (done (action agent3 term3)))))))").unwrap();
            }

            #[test]
            fn multiple_nested_predicates() {
                parse("((pred1 (pred2 term1 term2) (pred3 term3 term4)) (pred4 term5 term6))")
                    .unwrap();
            }

            #[test]
            fn complex_datetime() {
                parse("((action agent1 2023-05-17T10:30:00.123+02:00))").unwrap();
            }
        }
    }
}

mod serialize {
    use core::fmt::Write;

    use bstr::ByteSlice;

    use super::{AgentAction, Content, ContentElement, Predicate, Term};

    impl core::fmt::Display for Content {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_char('(')?;
            for e in self.0.iter() {
                write!(f, "{}", e)?;
            }
            f.write_char(')')
        }
    }

    impl core::fmt::Display for ContentElement {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use ContentElement::*;
            match self {
                AgentAction(a) => write!(f, "{}", a),
                Predicate(p) => write!(f, "{}", p),
            }
        }
    }

    impl core::fmt::Display for Predicate {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Predicate::Regular { symbol, terms } => {
                    if terms.len() != 0 {
                        f.write_char('(')?;
                    }
                    f.write_str(symbol.to_str().expect("symbol should be utf-8"))?;
                    for t in terms {
                        write!(f, " {}", t)?;
                    }
                    if terms.len() != 0 {
                        f.write_char(')')?;
                    }
                    Ok(())
                }
                Predicate::Result { lhs, rhs } => {
                    f.write_str("(result")?;
                    write!(f, " {}", lhs)?;
                    write!(f, " {}", rhs)?;
                    f.write_char(')')
                }
                Predicate::Done { action } => {
                    f.write_str("(done")?;
                    write!(f, " {}", action)?;
                    f.write_char(')')
                }
                Predicate::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
            }
        }
    }

    impl core::fmt::Display for AgentAction {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let Self { agent, action } = self;
            f.write_str("(action")?;
            write!(f, " {}", agent)?;
            write!(f, " {}", action)
        }
    }

    impl core::fmt::Display for Term {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            use Term::*;
            match self {
                Constant(c) => write!(f, "{}", c),
                Set(s) => {
                    f.write_str("(set")?;
                    for t in s.items.iter() {
                        write!(f, " {}", t)?;
                    }
                    f.write_char(')')
                }
                Sequence(s) => {
                    f.write_str("(sequence")?;
                    for t in s.items.iter() {
                        write!(f, " {}", t)?;
                    }
                    f.write_char(')')
                }
                Concept(c) => write!(f, "{}", c),
                Action(a) => write!(f, "{}", a),
            }
        }
    }
}

impl core::fmt::Display for Concept {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { symbol, parameters } = self;
        f.write_char('(')?;
        f.write_str(&symbol.to_str().expect("symbol should be utf-8"))?;
        match parameters {
            ConceptParameters::Positional(parameters) => {
                for p in parameters.iter() {
                    write!(f, " {}", p)?;
                }
            }
            ConceptParameters::ByName(parameters) => {
                for (n, v) in parameters.iter() {
                    write!(
                        f,
                        " {} {}",
                        n.to_str().expect("parameter name should be utf-8"),
                        v
                    )?;
                }
            }
        }
        f.write_char(')')
    }
}

impl core::fmt::Display for Constant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Constant::*;
        match self {
            Number(n) => write!(f, "{}", n),
            String(s) => {
                f.write_char('"')?;
                match s.to_str() {
                    Ok(s) => f.write_str(s)?,
                    Err(_) => unimplemented!(),
                }
                f.write_char('"')
            }
            Datatime(_) => todo!(),
        }
    }
}

impl core::fmt::Display for Number {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Number::*;
        match self {
            Int(i) => write!(f, "{}", i),
            Float(fl) => write!(f, "{}", fl),
        }
    }
}
