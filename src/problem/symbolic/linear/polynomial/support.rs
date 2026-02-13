use super::{LinearMonomial, LinearPolynomial, LinearSlice};
use crate::{
    domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
    problem::formula::FormulaId,
};

impl LinearPolynomial {
    pub fn empty(bound: RBound) -> Self {
        Self {
            linear_terms: Vec::new(),
            constant_term: ConcreteBitvector::new_zero(bound),
        }
    }

    pub fn from_monomial_and_constant(
        monomial: LinearMonomial,
        constant_term: ConcreteBitvector<RBound>,
    ) -> Self {
        Self {
            linear_terms: vec![monomial],
            constant_term,
        }
    }

    pub fn from_monomial(monomial: LinearMonomial) -> Self {
        let zero = ConcreteBitvector::new_zero(monomial.bound());
        Self::from_monomial_and_constant(monomial, zero)
    }

    pub fn from_concrete(constant: ConcreteBitvector<RBound>) -> Self {
        Self {
            linear_terms: Vec::new(),
            constant_term: constant,
        }
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        if let Some(slice) = LinearSlice::from_bounded(formula_id, bound) {
            let coefficient = ConcreteBitvector::new_one(bound);
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
            ConcreteBitvector::new_one(bound)
        } else {
            ConcreteBitvector::new_zero(bound)
        };

        LinearPolynomial::from_concrete(constant)
    }
}
