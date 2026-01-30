use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::linear::LinearCombination,
};

/// A linear relation `combination` <= `slack`.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearRelation {
    /// Left-side linear combination.
    pub combination: LinearCombination,
    /// Right-side slack value. With zero slack, the relation becomes equality.
    pub slack: ConcreteBitvector<RBound>,
}

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.combination, f)?;

        let op = if self.slack.is_zero() { "==" } else { "<=" };

        write!(f, " {} {}", op, self.slack)
    }
}
