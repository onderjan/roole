use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::LinearPolynomial;
use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

mod bitwise;
mod support;

/// A linear relation `polynomial` <= `slack`.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearRelation {
    /// Left-side linear polynomial.
    polynomial: LinearPolynomial,
    /// Right-side slack value. With zero slack, the relation becomes equality.
    slack: ConcreteBitvector<RBound>,
}

impl LinearRelation {
    pub fn from_eq(lhs: LinearPolynomial, rhs: LinearPolynomial) -> Self {
        // subtract polynomials and resolve equality to zero
        Self::from_eq_to_zero(lhs.sub(rhs))
    }

    pub fn from_eq_to_zero(polynomial: LinearPolynomial) -> Self {
        let zero = ConcreteBitvector::zero(polynomial.bound());
        Self {
            polynomial,
            slack: zero,
        }
    }

    pub(super) fn new(polynomial: LinearPolynomial, slack: ConcreteBitvector<RBound>) -> Self {
        assert_eq!(polynomial.bound(), slack.bound());
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
