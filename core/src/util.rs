pub(crate) mod time {
    #[cfg(target_os = "none")]
    pub(crate) use embassy_time::{Duration, Instant};
    #[cfg(not(target_os = "none"))]
    pub(crate) use std::time::{Duration, Instant};

    pub(crate) fn from_std_duration(duration: core::time::Duration) -> Duration {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "none")] {
                Duration::from_nanos(duration.as_nanos() as u64)
            } else {
                duration
            }
        }
    }
}

pub(crate) mod sync {
    use core::cell::Cell;

    #[repr(transparent)]
    pub(crate) struct AtomicU32(Cell<u32>);

    // SAFETY: Internal methods are protected using the [`critical-section`] crate.
    unsafe impl Sync for AtomicU32 {}

    impl AtomicU32 {
        pub(crate) const fn new(value: u32) -> Self {
            Self(Cell::new(value))
        }

        pub(crate) fn get_increment(&self) -> u32 {
            critical_section::with(|_| {
                let value = self.0.get();
                self.0
                    .replace(value.checked_add(1).expect("atomic u32 overflow"))
            })
        }
    }
}

pub(crate) mod parsing {
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
