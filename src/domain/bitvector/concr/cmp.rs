use std::cmp::Ordering;

use crate::domain::{bitvector::BitvectorBound, traits::forward::TypedCmp};

use super::ConcreteBitvector;

impl<B: BitvectorBound> TypedCmp for ConcreteBitvector<B> {
    type Output = ConcreteBitvector<B::SingleBit>;

    fn slt(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_signed() < rhs.into_signed();
        Self::Output::new(result as u64, B::single_bit_bound())
    }

    fn ult(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_unsigned() < rhs.into_unsigned();
        Self::Output::new(result as u64, B::single_bit_bound())
    }

    fn sle(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_signed() <= rhs.into_signed();
        Self::Output::new(result as u64, B::single_bit_bound())
    }

    fn ule(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_unsigned() <= rhs.into_unsigned();
        Self::Output::new(result as u64, B::single_bit_bound())
    }
}

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.bound, rhs.bound);
        self.to_u64().cmp(&rhs.to_u64())
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.bound, rhs.bound);
        self.to_i64().cmp(&rhs.to_i64())
    }
}
