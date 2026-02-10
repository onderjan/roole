use std::collections::BTreeMap;

use super::{LinearMonomial, LinearPolynomial, LinearSlice};
use crate::{
    domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
    problem::formula::FormulaId,
};

impl LinearPolynomial {
    pub fn new(
        linear_terms: BTreeMap<LinearSlice, ConcreteBitvector<RBound>>,
        constant_term: ConcreteBitvector<RBound>,
    ) -> Self {
        let result = Self {
            constant_term,
            linear_terms,
        };
        result.into_normal_form()
    }

    pub fn empty(bound: RBound) -> Self {
        Self {
            linear_terms: BTreeMap::new(),
            constant_term: ConcreteBitvector::zero(bound),
        }
    }

    pub fn from_monomial_and_constant(
        monomial: LinearMonomial,
        constant_term: ConcreteBitvector<RBound>,
    ) -> Self {
        Self::new(
            BTreeMap::from_iter([(monomial.slice, monomial.coefficient)]),
            constant_term,
        )
    }

    pub fn from_monomial(monomial: LinearMonomial) -> Self {
        Self::new(
            BTreeMap::from_iter([(monomial.slice, monomial.coefficient)]),
            ConcreteBitvector::zero(monomial.bound()),
        )
    }

    pub fn from_concrete(constant: ConcreteBitvector<RBound>) -> Self {
        Self {
            linear_terms: BTreeMap::new(),
            constant_term: constant,
        }
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        if let Some(slice) = LinearSlice::from_bounded(formula_id, bound) {
            let coefficient = ConcreteBitvector::one(bound);
            LinearPolynomial::from_monomial(LinearMonomial::new(coefficient, slice))
        } else {
            LinearPolynomial::empty(bound)
        }
    }

    pub fn bound(&self) -> RBound {
        self.constant_term.bound()
    }

    pub fn from_bool(constant: bool) -> LinearPolynomial {
        let bound = RBound::single_bit_bound();
        let constant = if constant {
            ConcreteBitvector::one(bound)
        } else {
            ConcreteBitvector::zero(bound)
        };

        LinearPolynomial::from_concrete(constant)
    }
}
