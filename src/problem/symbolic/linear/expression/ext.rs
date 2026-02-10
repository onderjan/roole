use super::LinearExpression;
use crate::domain::bitvector::RBound;

impl LinearExpression {
    pub fn uext(self, new_bound: RBound) -> Result<Self, ()> {
        // we can only zero-extend polynomials
        if let Self::Polynomial(polynomial) = self {
            Ok(Self::Polynomial(
                polynomial.uext(new_bound).map_err(|_| ())?,
            ))
        } else {
            Err(())
        }
    }

    pub fn sext(self, new_bound: RBound) -> Result<Self, ()> {
        // we can only sign-extend polynomials
        if let Self::Polynomial(polynomial) = self {
            Ok(Self::Polynomial(
                polynomial.sext(new_bound).map_err(|_| ())?,
            ))
        } else {
            Err(())
        }
    }
}
