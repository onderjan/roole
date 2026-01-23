use crate::domain::{bitvector::abstr::linear::LinearBitvector, traits::forward::HwShift};

impl HwShift for LinearBitvector {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        todo!()
    }

    fn logic_shr(self, amount: Self) -> Self {
        todo!()
    }

    fn arith_shr(self, amount: Self) -> Self {
        todo!()
    }
}
