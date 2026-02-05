use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId, operation::LinearPolynomial},
};

/// A linear relation `polynomial` <= `slack`.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearRelation {
    /// Left-side linear polynomial.
    polynomial: LinearPolynomial,
    /// Right-side slack value. With zero slack, the relation becomes equality.
    slack: ConcreteBitvector<RBound>,
}

impl LinearRelation {
    pub(super) fn new(polynomial: LinearPolynomial, slack: ConcreteBitvector<RBound>) -> Self {
        Self { polynomial, slack }
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let value = self.polynomial.evaluate(&fetch);
        let slack = D::single_value(*self.slack());

        // we are determining value <= slack
        value.ule(slack)
    }

    pub(super) fn polynomial(&self) -> &LinearPolynomial {
        &self.polynomial
    }

    pub(super) fn into_polynomial(self) -> LinearPolynomial {
        self.polynomial
    }

    pub(super) fn slack(&self) -> &ConcreteBitvector<RBound> {
        &self.slack
    }

    pub(super) fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        self.polynomial.remap(old_to_new);
    }

    pub(super) fn used_ids(&self) -> Vec<FormulaId> {
        self.polynomial.used_ids()
    }
}

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let one = ConcreteBitvector::one(self.polynomial.bound());
        if self.slack.add(one).is_full_mask() {
            // better to add 1 to the polynomial and print as non-equality
            let nonequality_polynomial = self
                .polynomial
                .clone()
                .add(LinearPolynomial::from_constant(one));
            Debug::fmt(&nonequality_polynomial, f)?;

            write!(f, " != 0")
        } else {
            Debug::fmt(&self.polynomial, f)?;

            let op = if self.slack.is_zero() { "==" } else { "<=" };

            write!(f, " {} {}", op, self.slack)
        }
    }
}
