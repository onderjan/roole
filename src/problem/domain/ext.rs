use crate::{
    domain::{bitvector::RBound, traits::forward::BExt},
    problem::domain::OperationDomain,
};

impl BExt<RBound> for OperationDomain {
    type Output = OperationDomain;

    fn uext(self, new_bound: RBound) -> Self::Output {
        let Ok(combination) = self.try_combination() else {
            return Self::Top(new_bound);
        };

        match combination.unsigned_extend(new_bound) {
            Ok(ok) => Self::from_combination(ok),
            Err(_) => Self::Top(new_bound),
        }
    }

    fn sext(self, new_bound: RBound) -> Self::Output {
        let Ok(combination) = self.try_combination() else {
            return Self::Top(new_bound);
        };

        match combination.signed_extend(new_bound) {
            Ok(ok) => Self::from_combination(ok),
            Err(_) => Self::Top(new_bound),
        }
    }
}
