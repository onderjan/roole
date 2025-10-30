use crate::domain::{bitvector::BitvectorBound, traits::forward::TypedEq};

use super::ConcreteBitvector;

impl<B: BitvectorBound> TypedEq for ConcreteBitvector<B> {
    type Output = ConcreteBitvector<B::SingleBit>;

    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value == rhs.value;
        Self::Output::new(result as u64, B::single_bit_bound())
    }

    fn ne(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value != rhs.value;
        Self::Output::new(result as u64, B::single_bit_bound())
    }
}
