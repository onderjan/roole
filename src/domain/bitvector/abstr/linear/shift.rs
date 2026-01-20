use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, linear::LinearBitvector},
        concr::ConcreteBitvector,
    },
    traits::forward::{Bitwise, HwShift},
};

impl<B: BitvectorBound> HwShift for LinearBitvector<B> {
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
