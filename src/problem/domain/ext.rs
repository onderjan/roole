use crate::{
    domain::{bitvector::RBound, traits::forward::BExt},
    problem::domain::LinearBitvector,
};

impl BExt<RBound> for LinearBitvector {
    type Output = LinearBitvector;

    fn uext(self, new_bound: RBound) -> Self::Output {
        // TODO: bit extension
        Self::Top(new_bound)
    }

    fn sext(self, new_bound: RBound) -> Self::Output {
        // TODO: bit extension
        Self::Top(new_bound)
    }
}
