use crate::domain::{bitvector::BitvectorBound, traits::forward::TypedEq};

use super::ConcreteBitvector;

impl<B: BitvectorBound> TypedEq for ConcreteBitvector<B> {
    type Output = ConcreteBitvector<B::SingleBit>;

    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value == rhs.value;
        Self::Output::from_bool(result, B::single_bit_bound())
    }

    fn ne(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value != rhs.value;
        Self::Output::from_bool(result, B::single_bit_bound())
    }

    fn ite(condition: Self::Output, then_branch: Self, else_branch: Self) -> Self {
        if condition.is_nonzero() {
            then_branch
        } else {
            else_branch
        }
    }
}
