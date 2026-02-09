use crate::{
    domain::{bitvector::RBound, traits::forward::BExt},
    problem::domain::OperationDomain,
};

impl BExt<RBound> for OperationDomain {
    type Output = OperationDomain;

    fn uext(self, new_bound: RBound) -> Self::Output {
        let Ok(polynomial) = self.try_into_polynomial() else {
            return Self::Top(new_bound);
        };

        match polynomial.uext(new_bound) {
            Ok(ok) => Self::from_polynomial(ok),
            Err(_) => Self::Top(new_bound),
        }
    }

    fn sext(self, new_bound: RBound) -> Self::Output {
        let Ok(polynomial) = self.try_into_polynomial() else {
            return Self::Top(new_bound);
        };

        match polynomial.sext(new_bound) {
            Ok(ok) => Self::from_polynomial(ok),
            Err(_) => Self::Top(new_bound),
        }
    }
}
