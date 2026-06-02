use alloc::borrow::Cow;
use alloc::boxed::Box;

use crate::message::{Message, Performative};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageFilter {
    Nested {
        negated: bool,
        operator: FilterOperator,
        filters: Box<[MessageFilter]>,
    },
    Literal {
        negated: bool,
        kind: FilterLiteral,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterLiteral {
    /// Matches with everything.
    All,
    /// Ontology name to match against.
    Ontology(Cow<'static, str>),
    /// Performative to match against.
    Performative(Performative),
}

impl MessageFilter {
    /// Creates a message filter that will match any message.
    pub const fn all() -> Self {
        Self::Literal {
            negated: false,
            kind: FilterLiteral::All,
        }
    }

    /// Creates a message filter that will not match any message.
    pub const fn none() -> Self {
        Self::all().negated()
    }

    /// Creates a message filter that will only match if the performative matches.
    pub const fn performative(performative: Performative) -> Self {
        Self::Literal {
            negated: false,
            kind: FilterLiteral::Performative(performative),
        }
    }

    pub fn ontology(ontology: impl Into<Cow<'static, str>>) -> Self {
        Self::Literal {
            negated: false,
            kind: FilterLiteral::Ontology(ontology.into()),
        }
    }
}

impl MessageFilter {
    /// Negates the output of given the message filter.
    pub const fn negated(mut self) -> Self {
        self.negate();
        self
    }

    /// Negates the output of the message filter.
    pub const fn negate(&mut self) {
        match self {
            Self::Nested { negated, .. } | Self::Literal { negated, .. } => *negated = !*negated,
        }
    }
}

impl MessageFilter {
    /// Creates a message filter that will only match if all of the given message filters
    /// match.
    pub fn and(filters: impl Into<Box<[Self]>>) -> Self {
        Self::nested(filters, FilterOperator::And, false)
    }

    /// Creates a message filter that will match if any of the given filters match.
    ///
    /// Note: This will stop matching once a single match is found.
    pub fn or(filters: impl Into<Box<[Self]>>) -> Self {
        Self::nested(filters, FilterOperator::Or, false)
    }

    fn nested(filters: impl Into<Box<[Self]>>, operator: FilterOperator, negated: bool) -> Self {
        let filters = filters.into();
        Self::Nested {
            negated,
            operator,
            filters,
        }
    }
}

impl MessageFilter {
    /// Checks if the filter matches with the provided message.
    pub fn matches(&self, message: &Message) -> bool {
        let matches = match self {
            Self::Nested {
                operator,
                filters,
                negated: _,
            } => match operator {
                FilterOperator::And => filters.iter().all(|f| f.matches(message)),
                FilterOperator::Or => filters.iter().any(|f| f.matches(message)),
            },
            Self::Literal { kind, negated: _ } => match kind {
                FilterLiteral::All => true,
                FilterLiteral::Ontology(mo) => {
                    message.ontology.as_ref().map(|o| mo == o).unwrap_or(false)
                }
                FilterLiteral::Performative(p) => message.performative == *p,
            },
        };
        matches ^ self.is_negated()
    }

    fn is_negated(&self) -> bool {
        match *self {
            Self::Nested { negated, .. } | Self::Literal { negated, .. } => negated,
        }
    }
}
