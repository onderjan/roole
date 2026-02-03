use std::collections::BTreeMap;

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::{BExt, HwArith},
    },
    problem::operation::LinearCombination,
};

impl LinearCombination {
    pub fn bit_not(self) -> Self {
        let mut result = self.arith_neg();
        result.constant = result.constant.sub(ConcreteBitvector::one(result.bound()));
        result.normalize();
        result
    }

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

    pub fn truncate(mut self, new_bound: RBound) -> Self {
        assert!(self.bound().width() > new_bound.width());

        // change constant term and coeff bounds

        self.constant = self.constant.uext(new_bound);

        for coeff in self.monomials.values_mut() {
            *coeff = coeff.uext(new_bound);
        }

        self.normalize();

        self
    }

    pub fn unsigned_extend(self, new_bound: RBound) -> Result<Self, Self> {
        let mut combination = match Self::try_shrink_or_identity(self, new_bound) {
            Ok(ok) => return Ok(ok),
            Err(combination) => combination,
        };

        // the new bound width is greater than old bound width
        // we will only extend if there had been definitely no overflow

        if combination.might_overflow() {
            // do not try anything
            return Err(combination);
        }
        // we know that we can extend the bounds
        // without breaking old overflow as it never happens

        combination.constant = combination.constant.uext(new_bound);

        for coeff in combination.monomials.values_mut() {
            *coeff = coeff.uext(new_bound);
        }

        Ok(combination)
    }

    pub fn signed_extend(self, new_bound: RBound) -> Result<Self, Self> {
        let combination = match Self::try_shrink_or_identity(self, new_bound) {
            Ok(ok) => return Ok(ok),
            Err(combination) => combination,
        };

        // TODO: perform signed extension
        Err(combination)
    }

    fn try_shrink_or_identity(combination: Self, new_bound: RBound) -> Result<Self, Self> {
        match new_bound.width().cmp(&combination.bound().width()) {
            std::cmp::Ordering::Less => {
                // the new bound is smaller than old bound
                // truncate
                Ok(combination.truncate(new_bound))
            }
            std::cmp::Ordering::Equal => {
                // no-op, the new bound is equal to old
                Ok(combination)
            }
            std::cmp::Ordering::Greater => Err(combination),
        }
    }
}
