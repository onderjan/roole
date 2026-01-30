use crate::{
    domain::{bitvector::RBound, traits::forward::BExt},
    problem::domain::OperationDomain,
};

impl BExt<RBound> for OperationDomain {
    type Output = OperationDomain;

    fn uext(self, new_bound: RBound) -> Self::Output {
        // TODO: bit extension
        Self::Top(new_bound)
    }

    fn sext(self, new_bound: RBound) -> Self::Output {
        // TODO: bit extension
        Self::Top(new_bound)
    }
}
