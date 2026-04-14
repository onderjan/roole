use crate::domain::{bitvector::BitvectorBound, traits::forward::Bitwise};

use super::ConcreteBitvector;

impl<B: BitvectorBound> Bitwise for ConcreteBitvector<B> {
    fn bit_not(self) -> Self {
        let value = self.value.uni_upwards(|a, _| (!a, ()), ());
        Self::from_masked(value, self.bound)
    }

    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let value = self.value.bi_upwards(rhs.value, |a, b, _| (a & b, ()), ());
        Self {
            bound: self.bound,
            value,
        }
    }

    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let value = self.value.bi_upwards(rhs.value, |a, b, _| (a | b, ()), ());
        Self {
            bound: self.bound,
            value,
        }
    }

    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let value = self.value.bi_upwards(rhs.value, |a, b, _| (a ^ b, ()), ());
        Self {
            bound: self.bound,
            value,
        }
    }
}
