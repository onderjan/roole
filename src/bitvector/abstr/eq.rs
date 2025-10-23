use crate::bitvector::abstr::RUnsigned;

use super::ThreeValued;

impl<T: RUnsigned> ThreeValued<T> {
    pub fn eq(self, rhs: Self, width: T::Width) -> Self {
        // result can be true if all bits can be the same
        // result can be false if at least one bit can be different

        let can_be_same_bits =
            (self.zeros.bitand(rhs.zeros, width)).bitor(self.ones.bitand(rhs.ones, width), width);
        let can_be_different_bits =
            (self.zeros.bitand(rhs.ones, width)).bitor(self.ones.bitand(rhs.zeros, width), width);

        let can_be_different = can_be_different_bits != T::zero(width);
        let can_be_same = can_be_same_bits == T::max_value(width);

        ThreeValued::from_bools(can_be_different, can_be_same)
    }

    pub fn ne(self, rhs: Self, width: T::Width) -> Self {
        self.eq(rhs, width).not(T::single_bit_width())
    }
}
