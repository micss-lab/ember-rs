mod parser {
    use alloc::boxed::Box;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;
    use core::marker::PhantomData;

    use chrono::{DateTime, FixedOffset};

    #[cfg(test)]
    extern crate std;

    /// List of expressions to form the content of an ACL message.
    #[derive(Debug, Clone, PartialEq)]
    struct Content(Vec<ContentExpression>);

    impl Content {
        pub fn parse(input: impl AsRef<bstr::BStr>) -> Result<Self, String> {
            sl0_content::content(&input::BStr::from(input.as_ref())).map_err(|e| e.to_string())
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    enum ContentExpression {
        Action(ActionExpression),
        Proposition(Proposition),
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Proposition(Wff);

    /// Well-formed formula.
    #[derive(Debug, Clone, PartialEq)]
    enum Wff {
        Atomic(AtomicFormula),
        Done { action: ActionExpression },
    }

    /// General collection type.
    ///
    /// Note: A set cannot be stored in a [`BTreeSet`] as it requires the
    /// items to be [`Ord`]. They cannot be as this would require evaluating
    /// the terms before storing them.
    ///
    /// [`BTreeSet`]: alloc::collections::btree_set::BTreeSet
    /// [`Ord`]: core::cmp::Ord
    #[derive(Debug, Clone, PartialEq)]
    struct Collection<C> {
        /// Items in the collection.
        items: Vec<()>,
        /// Semantics behind the collection.
        _marker: PhantomData<C>,
    }
    #[derive(Debug, Clone, PartialEq)]
    struct Set;
    #[derive(Debug, Clone, PartialEq)]
    struct Sequence;

    /// Numerical constant.
    #[derive(Debug, Clone, PartialEq)]
    enum Number {
        Int(i32),
        Float(f32),
    }

    /// Recursive structure defining the concept of a term.
    #[derive(Debug, Clone, PartialEq)]
    enum Term {
        Constant(Constant),
        Set(Collection<Set>),
        Sequence(Collection<Sequence>),
        Functional(FunctionalTerm),
        Action(Box<ActionExpression>),
    }

    /// Action (to be) performed by an agent.
    #[derive(Debug, Clone, PartialEq)]
    struct ActionExpression {
        /// Agent performing the action.
        agent: Term,
        /// Action being/to be performed.
        action: Term,
    }

    /// Indirect reference to an object via a functional relation with other objects.
    #[derive(Debug, Clone, PartialEq)]
    struct FunctionalTerm {
        symbol: bstr::BString,
        parameters: FunctionalParameters,
    }

    /// Parameters part of a functional term.
    #[derive(Debug, Clone, PartialEq)]
    enum FunctionalParameters {
        Positional(Vec<Term>),
        ByName(Vec<(bstr::BString, Term)>),
    }

    /// Atomic structures forming the building blocks of well-formed formula's.
    #[derive(Debug, Clone, PartialEq)]
    enum AtomicFormula {
        Proposition {
            symbol: bstr::BString,
        },
        /// The result of performing the action or evaluating the term denoted
        /// by the lhs is equal to the rhs.
        Result {
            lhs: Term,
            rhs: Term,
        },
        Predicate {
            /// Symbol identifying the predicate.
            symbol: bstr::BString,
            /// Terms passed to the predicate.
            terms: Vec<Term>,
        },
        Bool(bool),
    }

    /// Numerical, string, and time-related constants.
    #[derive(Debug, Clone, PartialEq)]
    enum Constant {
        Number(Number),
        String(bstr::BString),
        Datatime(DateTime<FixedOffset>),
    }

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
        grammar sl0_content<'a>() for input::BStr<'a> {
            use bstr::ByteSlice;
            use chrono::{DateTime, FixedOffset, Utc};

            rule _ = [c if c.is_ascii_whitespace()]?

            pub rule content() -> Content
                = lbrace() _ c:(c:content_expression() _ { c })+ _ rbrace() { Content(c) }

            rule content_expression() -> ContentExpression
                = a:action_expression() { ContentExpression::Action(a) }
                / p:proposition() { ContentExpression::Proposition(p) }

            rule proposition() -> Proposition
                = w:wff() { Proposition(w) }

            rule action_expression() -> ActionExpression
                = lbrace() _ "action" _ a:agent() _ t:term() _ rbrace() { ActionExpression { agent: a, action: t } }

            rule wff() -> Wff
                 = a:atomic_formula() { Wff::Atomic(a) }
                / lbrace() _ "done" _ a:action_expression() _ rbrace() { Wff::Done { action: a } }

            rule atomic_formula() -> AtomicFormula
                = s:propostion_symbol() { AtomicFormula::Proposition { symbol: s } }
                / lbrace() _ "result" _ lhs:term() _ rhs:term() _ rbrace() { AtomicFormula::Result { lhs, rhs } }
                / lbrace() _ s:predicate_symbol() _ t:(t:term() _ { t })+ _ rbrace() { AtomicFormula::Predicate { symbol: s, terms: t } }
                / "true" { AtomicFormula::Bool(true) }
                / "false" { AtomicFormula::Bool(false) }

            rule term() -> Term
                = c:constant() { Term::Constant(c) }
                / s:set() { Term::Set(s) }
                / s:sequence() { Term::Sequence(s) }
                / ft:functional_term() { Term::Functional(ft) }
                / ae:action_expression() { Term::Action(Box::new(ae)) }


            rule functional_term() -> FunctionalTerm
                = lbrace() _ s:function_symbol() _ p:(
                    t:(t:term() _ { t })* { FunctionalParameters::Positional(t) }
                    / p:(p:parameter() _ { p })* { FunctionalParameters::ByName(p) }
                  ) _ rbrace() { FunctionalTerm {symbol: s, parameters: p} }

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

            rule sequence() -> Collection<Sequence>
                = lbrace() _ "sequence" (term() _)* _ rbrace() { Collection { items: Vec::new(), _marker: PhantomData } }

            rule set() -> Collection<Set>
                = lbrace() _ "set" (term() _)* _ rbrace() { Collection { items: Vec::new(), _marker: PhantomData } }

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
                #[cfg(test)]
                {
                    use std::dbg;
                    dbg!("word: {}", s);
                }
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
        use alloc::string::String;

        fn parse(input: &str) -> Result<(), String> {
            use super::Content;
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
            parse("((action agent1 term1) (predicate term1 term2) (done (action agent2 term2)))")
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
            parse("((pred1 (pred2 term1 term2) (pred3 term3 term4)) (pred4 term5 term6))").unwrap();
        }

        #[test]
        fn complex_datetime() {
            parse("((action agent1 2023-05-17T10:30:00.123+02:00))").unwrap();
        }
    }
}
