use crate::domain::{
    bitvector::{BitvectorBound, abstr::linear::LinearBitvector},
    traits::forward::TypedCmp,
};

impl<B: BitvectorBound> TypedCmp for LinearBitvector<B> {
    type Output = LinearBitvector<B::SingleBit>;

    fn ult(self, rhs: Self) -> Self::Output {
        todo!()
    }

    fn ule(self, rhs: Self) -> Self::Output {
        todo!()
    }

    fn slt(self, rhs: Self) -> Self::Output {
        todo!()
    }

    fn sle(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
