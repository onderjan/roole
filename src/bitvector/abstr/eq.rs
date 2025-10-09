use crate::bitvector::abstr::Primitive;

use super::ThreeValued;

impl<T: Primitive> ThreeValued<T> {
    fn eq(self, rhs: Self, width: u32) -> Self {
        let width_mask = T::width_mask(width);

        // result can be true if all bits can be the same
        // result can be false if at least one bit can be different

        let can_be_same_bits = (self.zeros & rhs.zeros) | (self.ones & rhs.ones);
        let can_be_different_bits = (self.zeros & rhs.ones) | (self.ones & rhs.zeros);

        let can_be_different = can_be_different_bits != T::zero();
        let can_be_same = (can_be_same_bits & width_mask) == width_mask;

        ThreeValued::from_bools(can_be_different, can_be_same)
    }

    fn ne(self, rhs: Self, width: u32) -> Self {
        self.eq(rhs, width).bit_not(1)
    }
}
