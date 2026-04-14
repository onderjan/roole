use crate::domain::{bitvector::BitvectorBound, traits::forward::HwShift};

use super::ConcreteBitvector;

impl<B: BitvectorBound> HwShift for ConcreteBitvector<B> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        assert_eq!(self.bound, amount.bound);
        let value = self.value.unbounded_shl(amount.value);
        Self::from_masked(value, self.bound)
    }

    fn logic_shr(self, amount: Self) -> Self {
        assert_eq!(self.bound, amount.bound);
        let value = self.value.unbounded_shr(amount.value);
        Self::from_masked(value, self.bound)
    }

    fn arith_shr(self, amount: Self) -> Self {
        let bound = self.bound;
        assert_eq!(bound, amount.bound);

        let Some(hi) = bound.highest_bit() else {
            // zero width
            return self;
        };

        let sign_bit_set = self.is_sign_bit_set();

        let Some(amount_value) = amount.value.try_to_u32() else {
            // the shift much too big, fill with sign bit
            return if sign_bit_set {
                ConcreteBitvector::new_all_ones(bound)
            } else {
                ConcreteBitvector::new_zero(bound)
            };
        };

        if amount_value == 0 {
            // no shift
            return self;
        }

        // perform logical shift right
        let mut value = self.value.unbounded_shr(amount.value);

        // if the sign bit was set, set the shifted-in bits
        if sign_bit_set {
            let lo = hi.saturating_sub(amount_value - 1);
            value.set_bits(lo, hi, true);
        }

        Self::from_masked(value, bound)
    }
}
