use std::collections::BTreeMap;

use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::operation::LinearPolynomial,
};

impl LinearPolynomial {
    pub fn bit_not(self) -> Self {
        let mut result = self.arith_neg();
        result.constant_term = result
            .constant_term
            .sub(ConcreteBitvector::one(result.bound()));
        result.into_normal_form()
    }

    pub fn arith_neg(mut self) -> LinearPolynomial {
        self.constant_term = self.constant_term.arith_neg();
        for coefficient in self.linear_terms.values_mut() {
            *coefficient = (*coefficient).arith_neg();
        }

        self.into_normal_form()
    }

    pub fn add(self, rhs: LinearPolynomial) -> LinearPolynomial {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    pub fn sub(self, rhs: LinearPolynomial) -> LinearPolynomial {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn linear_combine(
        self,
        mut rhs: LinearPolynomial,
        op: fn(ConcreteBitvector<RBound>, ConcreteBitvector<RBound>) -> ConcreteBitvector<RBound>,
    ) -> LinearPolynomial {
        let constant = op(self.constant_term, rhs.constant_term);
        let mut monomials = BTreeMap::new();

        for (formula, left_coeff) in self.linear_terms {
            let coeff = if let Some(right_coeff) = rhs.linear_terms.remove(&formula) {
                op(left_coeff, right_coeff)
            } else {
                let zero = ConcreteBitvector::zero(left_coeff.bound());
                op(left_coeff, zero)
            };
            monomials.insert(formula, coeff);
        }

        for (formula, right_coeff) in rhs.linear_terms {
            let zero = ConcreteBitvector::zero(right_coeff.bound());
            let coeff = op(zero, right_coeff);
            monomials.insert(formula, coeff);
        }

        let polynomial = LinearPolynomial {
            constant_term: constant,
            linear_terms: monomials,
        };
        polynomial.into_normal_form()
    }
}
