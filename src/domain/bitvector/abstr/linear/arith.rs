use std::collections::BTreeMap;

use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{
            BitvectorDomain,
            linear::{LinearBitvector, LinearCombination},
        },
        concr::ConcreteBitvector,
    },
    traits::forward::HwArith,
};

impl<B: BitvectorBound> HwArith for LinearBitvector<B> {
    fn arith_neg(mut self) -> Self {
        let Some(combination) = &mut self.combination else {
            // already top value
            return self;
        };

        combination.constant = combination.constant.arith_neg();

        for coeff in combination.coefficients.values_mut() {
            *coeff = (*coeff).arith_neg();
        }

        self
    }
    fn add(self, rhs: Self) -> Self {
        linear_combine(self, rhs, |a, b| a.add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        linear_combine(self, rhs, |a, b| a.sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
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

fn linear_combine<B: BitvectorBound>(
    lhs: LinearBitvector<B>,
    rhs: LinearBitvector<B>,
    op: fn(ConcreteBitvector<B>, ConcreteBitvector<B>) -> ConcreteBitvector<B>,
) -> LinearBitvector<B> {
    assert_eq!(lhs.bound, rhs.bound);
    let bound = lhs.bound;

    let (Some(lhs), Some(mut rhs)) = (lhs.combination, rhs.combination) else {
        return LinearBitvector::<B>::top(bound);
    };

    let constant = lhs.constant.add(rhs.constant);
    let mut coefficients = BTreeMap::new();

    for (formula, left_coeff) in lhs.coefficients {
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

    LinearBitvector {
        bound,
        combination: Some(LinearCombination {
            constant,
            coefficients,
        }),
    }
}
