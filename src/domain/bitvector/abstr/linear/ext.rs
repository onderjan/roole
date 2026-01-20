use crate::domain::{
    bitvector::{BitvectorBound, CBound, abstr::linear::LinearBitvector},
    traits::forward::{BExt, Ext},
};

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for LinearBitvector<B> {
    type Output = LinearBitvector<X>;

    fn uext(self, new_bound: X) -> Self::Output {
        todo!()
    }

    fn sext(self, new_bound: X) -> Self::Output {
        todo!()
    }
}

impl<const W: u32, const X: u32> Ext<X> for LinearBitvector<CBound<W>> {
    type Output = LinearBitvector<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
