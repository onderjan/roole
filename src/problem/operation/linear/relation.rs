use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId, operation::LinearCombination},
};

/// A linear relation `combination` <= `slack`.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearRelation {
    /// Left-side linear combination.
    combination: LinearCombination,
    /// Right-side slack value. With zero slack, the relation becomes equality.
    slack: ConcreteBitvector<RBound>,
}

impl LinearRelation {
    pub(super) fn new(combination: LinearCombination, slack: ConcreteBitvector<RBound>) -> Self {
        Self { combination, slack }
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let value = self.combination.evaluate(&fetch);
        let slack = D::single_value(*self.slack());

        // we are determining value <= slack
        value.ule(slack)
    }

    pub(super) fn combination(&self) -> &LinearCombination {
        &self.combination
    }

    pub(super) fn slack(&self) -> &ConcreteBitvector<RBound> {
        &self.slack
    }

    pub(super) fn remap(&mut self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) {
        self.combination.remap(old_to_new);
    }

    pub(super) fn used_ids(&self) -> Vec<FormulaId> {
        self.combination.used_ids()
    }
}

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.combination, f)?;

        let op = if self.slack.is_zero() { "==" } else { "<=" };

        write!(f, " {} {}", op, self.slack)
    }
}
