use crate::bitvector::abstr::{RUnsigned, ThreeValued};

impl<T: RUnsigned> ThreeValued<T> {
    pub fn new(value: T, width: T::Width) -> Self {
        let zeros = value.not(width);
        let ones = value;

        Self::from_zeros_ones(zeros, ones, width)
    }

    pub(super) fn from_zeros_ones(zeros: T, ones: T, width: T::Width) -> Self {
        assert_eq!(zeros, zeros.limited(width));
        assert_eq!(ones, ones.limited(width));

        Self { zeros, ones }
    }

    pub(super) fn from_bools(zeros: bool, ones: bool) -> Self {
        let bool_width = T::single_bit_width();
        let false_value = T::zero(bool_width);
        let true_value = T::max_value(bool_width);

        let zeros = if zeros { true_value } else { false_value };
        let ones = if ones { true_value } else { false_value };

        Self { zeros, ones }
    }

    pub fn new_unknown(width: T::Width) -> Self {
        let max_value = T::max_value(width);

        Self {
            zeros: max_value,
            ones: max_value,
        }
    }

    #[must_use]
    pub fn umin(&self, width: T::Width) -> T {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        (self.zeros.not(width)).limited(width)
    }

    #[must_use]
    pub fn umax(&self, _width: T::Width) -> T {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones
    }

    /*#[must_use]
    pub fn smin(&self, width: T::Width) -> T::Signed {
        let sign_bit_mask = T::sign_bit_mask(width);
        // take the unsigned minimum
        let mut result = self.umin(width);
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set(width) {
            result = result | sign_bit_mask
        }
        result.cast_signed(width)
    }

    #[must_use]
    pub fn smax(&self, width: T::Width) -> T::Signed {
        let sign_bit_mask = T::sign_bit_mask(width);
        // take the unsigned maximum
        let mut result = self.umax(width);
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set(width) {
            result = result & !sign_bit_mask;
        }
        result.cast_signed(width)
    }*/

    /*pub fn is_zeros_sign_bit_set(&self, width: u32) -> bool {
        let sign_bit_mask = T::sign_bit_mask(width);
        (self.zeros & sign_bit_mask) != T::zero()
    }

    pub fn is_ones_sign_bit_set(&self, width: u32) -> bool {
        let sign_bit_mask = T::sign_bit_mask(width);
        (self.ones & sign_bit_mask) != T::zero()
    }*/
}
