use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
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

    pub(super) fn into_combination(self) -> LinearCombination {
        self.combination
    }

    pub(super) fn slack(&self) -> &ConcreteBitvector<RBound> {
        &self.slack
    }

    pub(super) fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        self.combination.remap(old_to_new);
    }

    pub(super) fn used_ids(&self) -> Vec<FormulaId> {
        self.combination.used_ids()
    }
}

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let one = ConcreteBitvector::one(self.combination.bound());
        if self.slack.add(one).is_full_mask() {
            // better to add 1 to the combination and print as non-equality
            let nonequality_combination = self
                .combination
                .clone()
                .add(LinearCombination::from_constant(one));
            Debug::fmt(&nonequality_combination, f)?;

            write!(f, " != 0")
        } else {
            Debug::fmt(&self.combination, f)?;

            let op = if self.slack.is_zero() { "==" } else { "<=" };

            write!(f, " {} {}", op, self.slack)
        }
    }
}
