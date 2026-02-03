use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound},
        traits::forward::BExt,
    },
    problem::operation::LinearCombination,
};

impl LinearCombination {
    pub fn uext(self, new_bound: RBound) -> Result<Self, Self> {
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

    pub fn sext(self, new_bound: RBound) -> Result<Self, Self> {
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

    fn truncate(mut self, new_bound: RBound) -> Self {
        assert!(self.bound().width() > new_bound.width());

        // change constant term and coeff bounds

        self.constant = self.constant.uext(new_bound);

        for coeff in self.monomials.values_mut() {
            *coeff = coeff.uext(new_bound);
        }

        self.normalize();

        self
    }
}
