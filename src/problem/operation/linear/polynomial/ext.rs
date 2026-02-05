use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound},
        traits::forward::BExt,
    },
    problem::operation::LinearPolynomial,
};

impl LinearPolynomial {
    pub fn uext(self, new_bound: RBound) -> Result<Self, Self> {
        let mut polynomial = match Self::try_shrink_or_identity(self, new_bound) {
            Ok(ok) => return Ok(ok),
            Err(polynomial) => polynomial,
        };

        // the new bound width is greater than old bound width
        // we will only extend if there had been definitely no overflow

        if polynomial.might_overflow() {
            // do not try anything
            return Err(polynomial);
        }
        // we know that we can extend the bounds
        // without breaking old overflow as it never happens

        polynomial.constant = polynomial.constant.uext(new_bound);

        for coeff in polynomial.monomials.values_mut() {
            *coeff = coeff.uext(new_bound);
        }

        Ok(polynomial)
    }

    pub fn sext(self, new_bound: RBound) -> Result<Self, Self> {
        let polynomial = match Self::try_shrink_or_identity(self, new_bound) {
            Ok(ok) => return Ok(ok),
            Err(polynomial) => polynomial,
        };

        // TODO: perform signed extension
        Err(polynomial)
    }

    fn try_shrink_or_identity(polynomial: Self, new_bound: RBound) -> Result<Self, Self> {
        match new_bound.width().cmp(&polynomial.bound().width()) {
            std::cmp::Ordering::Less => {
                // the new bound is smaller than old bound
                // truncate
                Ok(polynomial.truncate(new_bound))
            }
            std::cmp::Ordering::Equal => {
                // no-op, the new bound is equal to old
                Ok(polynomial)
            }
            std::cmp::Ordering::Greater => Err(polynomial),
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
