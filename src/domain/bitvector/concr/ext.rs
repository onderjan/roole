use crate::domain::{
    bitvector::{BitvectorBound, CBound},
    traits::forward::{BExt, Ext},
};

use super::ConcreteBitvector;

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for ConcreteBitvector<B> {
    type Output = ConcreteBitvector<X>;

    fn uext(self, new_bound: X) -> ConcreteBitvector<X> {
        // shorten or lengthen as needed
        ConcreteBitvector::from_masked(self.value, new_bound)
    }

    fn sext(self, new_bound: X) -> ConcreteBitvector<X> {
        let old_width = self.bound.width();
        let new_width = new_bound.width();
        let should_set_sign_bit = new_width > old_width && self.is_sign_bit_set();
        // shorten or lengthen as needed
        let mut result = ConcreteBitvector::from_masked(self.value, new_bound);
        // set the sign bit in higher positions if needed
        if should_set_sign_bit {
            let num_set_bits = new_width - old_width;
            if let Some(hi) = new_bound.highest_bit() {
                let lo = hi.saturating_sub(num_set_bits - 1);
                result.set_bits(lo, hi, true);
            }
        }

        result
    }
}

impl<const W: u32, const X: u32> Ext<X> for ConcreteBitvector<CBound<W>> {
    type Output = ConcreteBitvector<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
