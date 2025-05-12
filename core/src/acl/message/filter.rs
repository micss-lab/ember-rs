use alloc::vec::Vec;
use core::ops::Not;

use super::{Message, Ontology, Performative};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageFilter {
    Nested {
        negated: bool,
        operator: FilterOperator,
        filters: Vec<MessageFilter>,
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
    Ontology(Ontology),
    /// Performative to match against.
    Performative(Performative),
}

impl MessageFilter {
    /// Creates a message filter that will match any message.
    pub const fn all() -> MessageFilter {
        MessageFilter::Literal {
            negated: false,
            kind: FilterLiteral::All,
        }
    }

    /// Creates a message filter that will not match any message.
    pub fn none() -> MessageFilter {
        MessageFilter::all().negated()
    }

    /// Creates a message filter that will only match if the performative matches.
    pub fn performative(performative: Performative) -> MessageFilter {
        MessageFilter::Literal {
            negated: false,
            kind: FilterLiteral::Performative(performative),
        }
    }

    pub fn ontology(ontology: Ontology) -> MessageFilter {
        MessageFilter::Literal {
            negated: false,
            kind: FilterLiteral::Ontology(ontology),
        }
    }
}

impl MessageFilter {
    /// Creates a message filter that will only match if all of the given message filters
    /// match.
    pub fn and(filters: impl Into<Vec<MessageFilter>>) -> MessageFilter {
        Self::nested(filters, FilterOperator::And, false)
    }

    /// Creates a message filter that will match if any of the given filters match.
    ///
    /// Note: This will stop matching once a single match is found.
    pub fn or(filters: impl Into<Vec<MessageFilter>>) -> MessageFilter {
        Self::nested(filters, FilterOperator::Or, false)
    }

    /// Negates the output of given the message filter.
    pub fn negated(mut self) -> MessageFilter {
        self.negate();
        self
    }

    /// Negates the output of the message filter.
    pub fn negate(&mut self) {
        match self {
            MessageFilter::Nested { negated, .. } | MessageFilter::Literal { negated, .. } => {
                *negated = negated.not()
            }
        }
    }

    fn nested(
        filters: impl Into<Vec<MessageFilter>>,
        operator: FilterOperator,
        negated: bool,
    ) -> Self {
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
            MessageFilter::Nested {
                operator,
                filters,
                negated: _,
            } => match operator {
                FilterOperator::And => filters.iter().all(|f| f.matches(message)),
                FilterOperator::Or => filters.iter().any(|f| f.matches(message)),
            },
            MessageFilter::Literal { kind, negated: _ } => match kind {
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
            MessageFilter::Nested { negated, .. } | MessageFilter::Literal { negated, .. } => {
                negated
            }
        }
    }
}
