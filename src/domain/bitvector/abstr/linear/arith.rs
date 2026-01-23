use std::collections::BTreeMap;

use crate::domain::{
    bitvector::{
        RBound,
        abstr::{
            BitvectorDomain,
            linear::{LinearBitvector, LinearCombination, LinearType},
        },
        concr::ConcreteBitvector,
    },
    traits::forward::HwArith,
};

impl HwArith for LinearBitvector {
    fn arith_neg(self) -> Self {
        let LinearType::Combination(combination) = self.ty else {
            // return top value
            return Self::top(self.bound);
        };

        Self {
            bound: self.bound,
            ty: LinearType::Combination(combination.arith_neg()),
        }
    }
    fn add(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
        // TODO: multiply if one has a definite value
        todo!()
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
        assert_eq!(self.bound, rhs.bound);
        let bound = self.bound;

        let (LinearType::Combination(lhs), LinearType::Combination(rhs)) = (self.ty, rhs.ty) else {
            return LinearBitvector::top(bound);
        };

        let combination = op(lhs, rhs);

        Self {
            bound,
            ty: LinearType::Combination(combination),
        }
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
