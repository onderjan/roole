use std::collections::BTreeMap;

use super::LinearPolynomial;
use crate::domain::{
    bitvector::{RBound, concr::ConcreteBitvector},
    traits::forward::HwArith,
};

impl LinearPolynomial {
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

    pub fn mul(self, rhs: LinearPolynomial) -> Result<LinearPolynomial, ()> {
        // we can only multiply if at least one of the polynomials is constant
        let (constant, mut polynomial) = if let Some(constant) = self.constant_value() {
            (constant, rhs)
        } else if let Some(constant) = rhs.constant_value() {
            (constant, self)
        } else {
            // neither is a constant
            return Err(());
        };

        // multiply polynomial by constant
        polynomial.scale(constant);
        Ok(polynomial)
    }

    pub fn scale(&mut self, scaler: ConcreteBitvector<RBound>) {
        let bound = self.bound();
        assert_eq!(bound, scaler.bound());

        self.constant_term = self.constant_term.mul(scaler);

        for coefficient in self.linear_terms.values_mut() {
            *coefficient = coefficient.mul(scaler);
        }
    }

    pub fn linear_combine(
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

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, num::NonZero};

    use crate::{
        domain::bitvector::{RBound, concr::ConcreteBitvector},
        problem::formula::{FormulaId, VariableId},
    };

    use super::*;

    use super::super::LinearSlice;

    #[test]
    fn test_addsub() {
        let bound = RBound::new(32);
        let slice = LinearSlice {
            formula_id: FormulaId::Variable(VariableId(0)),
            lsb: 0,
            width: NonZero::new(32).unwrap(),
        };
        let a = LinearPolynomial {
            constant_term: ConcreteBitvector::new(38, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(12, bound))]),
        };
        let b = LinearPolynomial {
            constant_term: ConcreteBitvector::new(17, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(7, bound))]),
        };
        let add_result = LinearPolynomial {
            constant_term: ConcreteBitvector::new(55, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(19, bound))]),
        };
        let sub_result = LinearPolynomial {
            constant_term: ConcreteBitvector::new(21, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(5, bound))]),
        };
        assert_eq!(a.clone().add(b.clone()), add_result);
        assert_eq!(a.sub(b), sub_result);
    }
}
