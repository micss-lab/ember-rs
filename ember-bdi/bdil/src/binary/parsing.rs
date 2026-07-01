pub(super) struct Bytes<'a>(pub &'a [u8]);

impl<'a> From<&'a [u8]> for Bytes<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }
}

impl<'a> core::ops::Deref for Bytes<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.0
    }
}

pub(super) struct ByteOffset(pub usize);

impl core::fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "byte {}", self.0)
    }
}

impl<'a> peg::Parse for Bytes<'a> {
    type PositionRepr = ByteOffset;

    fn start(&self) -> usize {
        0
    }

    fn is_eof(&self, pos: usize) -> bool {
        pos >= self.0.len()
    }

    fn position_repr(&self, pos: usize) -> ByteOffset {
        ByteOffset(pos)
    }
}

impl<'a, 'input> peg::ParseElem<'input> for Bytes<'a> {
    type Element = u8;

    fn parse_elem(&'input self, pos: usize) -> peg::RuleResult<u8> {
        match self.0.get(pos) {
            Some(&b) => peg::RuleResult::Matched(pos + 1, b),
            None => peg::RuleResult::Failed,
        }
    }
}

impl<'a> peg::ParseLiteral for Bytes<'a> {
    fn parse_string_literal(&self, pos: usize, literal: &str) -> peg::RuleResult<()> {
        let l = literal.len();
        if self.0.len() >= pos + l && &self.0[pos..pos + l] == literal.as_bytes() {
            peg::RuleResult::Matched(pos + l, ())
        } else {
            peg::RuleResult::Failed
        }
    }
}

impl<'a, 'input> peg::ParseSlice<'input> for Bytes<'a> {
    type Slice = &'input [u8];

    fn parse_slice(&'input self, p1: usize, p2: usize) -> &'input [u8] {
        &self.0[p1..p2]
    }
}
