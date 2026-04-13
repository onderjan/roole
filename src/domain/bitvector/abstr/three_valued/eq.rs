use crate::domain::{
    bitvector::BitvectorBound,
    traits::{
        Join,
        forward::{Bitwise, TypedEq},
    },
    value::ThreeValued,
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> TypedEq for ThreeValuedBitvector<B> {
    type Output = ThreeValuedBitvector<B::SingleBit>;
    fn eq(self, rhs: Self) -> Self::Output {
        // result can be false if at least one bit can be different
        // result can be true if all bits can be the same

        let can_be_different_bits = (self.zeros.clone().bit_and(rhs.ones.clone()))
            .bit_or(self.ones.clone().bit_and(rhs.zeros.clone()));
        let can_be_same_bits = (self.zeros.bit_and(rhs.zeros)).bit_or(self.ones.bit_and(rhs.ones));

        let can_be_different = can_be_different_bits.is_nonzero();
        let can_be_same = can_be_same_bits.is_full_mask();

        Self::Output::from_bools(can_be_different, can_be_same)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }

    fn ite(condition: Self::Output, then_branch: Self, else_branch: Self) -> Self {
        match condition.three_valued_from_bit(0) {
            ThreeValued::False => else_branch,
            ThreeValued::True => then_branch,
            ThreeValued::Unknown => then_branch.join(&else_branch),
        }
    }
}
