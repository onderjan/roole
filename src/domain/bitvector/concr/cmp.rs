use std::cmp::Ordering;

use crate::domain::{bitvector::BitvectorBound, traits::forward::TypedCmp};

use super::ConcreteBitvector;

impl<B: BitvectorBound> TypedCmp for ConcreteBitvector<B> {
    type Output = ConcreteBitvector<B::SingleBit>;

    fn slt(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_signed() < rhs.into_signed();
        Self::Output::from_bool(result, B::single_bit_bound())
    }

    fn ult(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_unsigned() < rhs.into_unsigned();
        Self::Output::from_bool(result, B::single_bit_bound())
    }

    fn sle(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_signed() <= rhs.into_signed();
        Self::Output::from_bool(result, B::single_bit_bound())
    }

    fn ule(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.into_unsigned() <= rhs.into_unsigned();
        Self::Output::from_bool(result, B::single_bit_bound())
    }
}

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.bound, rhs.bound);
        self.value.unsigned_cmp(&rhs.value)
    }
    pub fn signed_cmp(&self, rhs: &Self) -> Ordering {
        assert_eq!(self.bound, rhs.bound);
        let lhs_sign = self.is_sign_bit_set();
        let rhs_sign = rhs.is_sign_bit_set();
        if lhs_sign != rhs_sign {
            if lhs_sign {
                // lhs negative, rhs non-negative: lhs < rhs
                return Ordering::Less;
            } else {
                // lhs non-negative, rhs negative: lhs > rhs
                return Ordering::Greater;
            }
        }
        // both have the same sign, we can do an unsigned comparison
        self.unsigned_cmp(rhs)
    }
}
