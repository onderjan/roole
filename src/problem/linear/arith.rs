use std::collections::BTreeMap;

use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::linear::LinearCombination,
};

impl LinearCombination {
    pub fn arith_neg(mut self) -> LinearCombination {
        self.constant = self.constant.arith_neg();
        for coefficient in self.monomials.values_mut() {
            *coefficient = (*coefficient).arith_neg();
        }

        self.normalize();

        self
    }

    pub fn add(self, rhs: LinearCombination) -> LinearCombination {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    pub fn sub(self, rhs: LinearCombination) -> LinearCombination {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn linear_combine(
        self,
        mut rhs: LinearCombination,
        op: fn(ConcreteBitvector<RBound>, ConcreteBitvector<RBound>) -> ConcreteBitvector<RBound>,
    ) -> LinearCombination {
        let constant = op(self.constant, rhs.constant);
        let mut monomials = BTreeMap::new();

        for (formula, left_coeff) in self.monomials {
            let coeff = if let Some(right_coeff) = rhs.monomials.remove(&formula) {
                op(left_coeff, right_coeff)
            } else {
                let zero = ConcreteBitvector::zero(left_coeff.bound());
                op(left_coeff, zero)
            };
            monomials.insert(formula, coeff);
        }

        for (formula, right_coeff) in rhs.monomials {
            let zero = ConcreteBitvector::zero(right_coeff.bound());
            let coeff = op(zero, right_coeff);
            monomials.insert(formula, coeff);
        }

        let mut combination = LinearCombination {
            constant,
            monomials,
        };
        combination.normalize();

        combination
    }
}
