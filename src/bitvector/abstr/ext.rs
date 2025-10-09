use crate::bitvector::abstr::Primitive;

use super::ThreeValued;

impl<T: Primitive> ThreeValued<T> {
    fn uext(self, old_width: u32, new_width: u32) -> Self {
        let old_mask = T::width_mask(old_width);
        let new_mask = T::width_mask(new_width);

        // shorten if needed
        let shortened_zeros = self.zeros & new_mask;
        let shortened_ones = self.ones & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to add zeros
        let zeros = shortened_zeros | lengthening_mask;
        let ones = shortened_ones;

        // shorten if needed, lengthening is fine
        Self::from_zeros_ones(zeros, ones, new_width)
    }

    fn sext(self, old_width: u32, new_width: u32) -> Self {
        let old_mask = T::width_mask(old_width);
        let new_mask = T::width_mask(new_width);

        // shorten if needed
        let shortened_zeros = self.zeros & new_mask;
        let shortened_ones = self.ones & new_mask;

        // the mask for lengthening is comprised of bits
        // that were not in the old mask but are in the new mask
        let lengthening_mask = !old_mask & new_mask;

        // for lengthening, we need to extend whatever may be in the sign bit
        let zeros = if self.is_zeros_sign_bit_set(old_width) {
            shortened_zeros | lengthening_mask
        } else {
            shortened_zeros
        };

        let ones = if self.is_ones_sign_bit_set(old_width) {
            shortened_ones | lengthening_mask
        } else {
            shortened_ones
        };

        Self::from_zeros_ones(zeros, ones, new_width)
    }
}
