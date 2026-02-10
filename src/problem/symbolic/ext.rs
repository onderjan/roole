use super::SymbolicDomain;
use crate::domain::{bitvector::RBound, traits::forward::BExt};

impl BExt<RBound> for SymbolicDomain {
    type Output = SymbolicDomain;

    fn uext(self, new_bound: RBound) -> Self::Output {
        // just try to resolve in linear
        let SymbolicDomain::Linear(linear) = self else {
            return Self::Top(new_bound);
        };

        let result = linear.uext(new_bound);
        // be careful to have the result with the new bound
        result
            .map(Self::Linear)
            .unwrap_or(SymbolicDomain::Top(new_bound))
    }
    fn sext(self, new_bound: RBound) -> Self::Output {
        // just try to resolve in linear
        let SymbolicDomain::Linear(linear) = self else {
            return Self::Top(new_bound);
        };

        let result = linear.sext(new_bound);
        // be careful to have the result with the new bound
        result
            .map(Self::Linear)
            .unwrap_or(SymbolicDomain::Top(new_bound))
    }
}
