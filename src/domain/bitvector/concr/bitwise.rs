use crate::domain::{bitvector::BitvectorBound, traits::forward::Bitwise};

use super::ConcreteBitvector;

impl<B: BitvectorBound> Bitwise for ConcreteBitvector<B> {
    fn bit_not(self) -> Self {
        Self::from_masked(!self.value, self.bound)
    }
    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self {
            bound: self.bound,
            value: self.value & rhs.value,
        }
    }
    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self {
            bound: self.bound,
            value: self.value | rhs.value,
        }
    }
    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self {
            bound: self.bound,
            value: self.value ^ rhs.value,
        }
    }
}
