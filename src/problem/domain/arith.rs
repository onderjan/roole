use std::collections::BTreeMap;

use crate::{
    domain::{
        bitvector::{RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::domain::{LinearBitvector, LinearCombination},
};

impl HwArith for LinearBitvector {
    fn arith_neg(self) -> Self {
        let LinearBitvector::Combination(combination) = self else {
            // return top value
            return Self::top(self.bound());
        };

        LinearBitvector::Combination(combination.arith_neg())
    }
    fn add(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (LinearBitvector::Combination(lhs), LinearBitvector::Combination(rhs)) = (self, rhs)
        else {
            // return top value
            return Self::top(bound);
        };

        let (constant, mut combination) = if lhs.coefficients.is_empty() {
            (lhs.constant, rhs)
        } else if rhs.coefficients.is_empty() {
            (rhs.constant, lhs)
        } else {
            // return top value
            return Self::top(bound);
        };

        // multiply combination by constant
        combination.apply_fixed_mult(constant);
        Self::Combination(combination)
    }

    fn udiv(self, _rhs: Self) -> Self {
        todo!("udiv")
    }

    fn sdiv(self, _rhs: Self) -> Self {
        todo!("sdiv")
    }

    fn urem(self, _rhs: Self) -> Self {
        todo!("urem")
    }

    fn srem(self, _rhs: Self) -> Self {
        todo!("srem")
    }
}

impl LinearBitvector {
    fn linear_combine(
        self,
        rhs: LinearBitvector,
        op: fn(LinearCombination, LinearCombination) -> LinearCombination,
    ) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (LinearBitvector::Combination(lhs), LinearBitvector::Combination(rhs)) = (self, rhs)
        else {
            return LinearBitvector::top(bound);
        };

        let combination = op(lhs, rhs);

        LinearBitvector::Combination(combination)
    }
}

impl LinearCombination {
    pub(super) fn arith_neg(mut self) -> LinearCombination {
        self.constant = self.constant.arith_neg();
        for coeff in self.coefficients.values_mut() {
            *coeff = (*coeff).arith_neg();
        }

        self.normalize();

        self
    }

    pub(super) fn add(self, rhs: LinearCombination) -> LinearCombination {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    pub(super) fn sub(self, rhs: LinearCombination) -> LinearCombination {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn linear_combine(
        self,
        mut rhs: LinearCombination,
        op: fn(ConcreteBitvector<RBound>, ConcreteBitvector<RBound>) -> ConcreteBitvector<RBound>,
    ) -> LinearCombination {
        let constant = self.constant.add(rhs.constant);
        let mut coefficients = BTreeMap::new();

        for (formula, left_coeff) in self.coefficients {
            let coeff = if let Some(right_coeff) = rhs.coefficients.remove(&formula) {
                op(left_coeff, right_coeff)
            } else {
                let zero = ConcreteBitvector::zero(left_coeff.bound());
                op(left_coeff, zero)
            };
            coefficients.insert(formula, coeff);
        }

        for (formula, right_coeff) in rhs.coefficients {
            let zero = ConcreteBitvector::zero(right_coeff.bound());
            let coeff = op(zero, right_coeff);
            coefficients.insert(formula, coeff);
        }

        let mut combination = LinearCombination {
            constant,
            coefficients,
        };
        combination.normalize();

        combination
    }
}
