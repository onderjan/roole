use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{Debug, UpperHex},
};

use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::{Bitwise, HwArith},
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

    pub(super) fn bit_not(self) -> Result<Self, LinearPolynomial> {
        // consider modulus 'm', left side 'a' and right side slack 's'
        // where 0 <= a < m, 0 <= s < m
        // we can now manipulate inequalities without regard to modularity
        // as long as we ensure the end values are within [0, m-1]
        // we want to negate the original inequality !(a <= s) and obtain the same lesser-or-equal form
        // 1. propagate negation into inequality: a > s
        // 2. multiply by minus one: -a < -s
        // 3. add m to both sides: m-a < m-s
        // 4. subtract 1 from right side and change to non-strict inequality: m-a <= m-s-1
        // 5. to bring the left side into bounds, subtract 1 from both sides: m-a-1 <= m-s-2
        // 6. use (!x) = m-x-1 to simplify: (!a) <= (!s)-1
        // for left side, 0 <= (!a) < m, but for right side, -1 <= (!s)-1 < m-1
        // handle the case where (!s) == 0 specially

        let bit_not_slack = self.slack().bit_not();
        if bit_not_slack.is_zero() {
            // the relation a <= s was a tautology as s was the highest possible value
            // return contradiction
            return Err(LinearPolynomial::single_bit(false));
        }

        // we now know 0 <= (!a) < m and 0 <= (!s)-1 < m-1
        // as such, we can construct the relation -a <= (!s-1)
        // as the negation of a <= s

        let polynomial = self.polynomial.clone().bit_not();
        let slack = bit_not_slack.sub(ConcreteBitvector::one(self.slack.bound()));

        Ok(LinearRelation::new(polynomial, slack))
    }

    pub(super) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
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

            write!(f, " {} ", op)?;

            if hex {
                write!(f, "{:#X}", self.slack)
            } else {
                write!(f, "{:?}", self.slack)
            }
        }
    }
}

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
