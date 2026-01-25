use crate::domain::{
    bitvector::{RBound, abstr::linear::LinearBitvector},
    traits::forward::BExt,
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
