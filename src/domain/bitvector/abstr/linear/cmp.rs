use crate::domain::{bitvector::abstr::linear::LinearBitvector, traits::forward::TypedCmp};

impl TypedCmp for LinearBitvector {
    type Output = LinearBitvector;

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
