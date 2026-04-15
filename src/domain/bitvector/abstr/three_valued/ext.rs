use crate::domain::{
    bitvector::{BitvectorBound, CBound, abstr::BitvectorDomain},
    traits::forward::{BExt, Ext},
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound, X: BitvectorBound> BExt<X> for ThreeValuedBitvector<B> {
    type Output = ThreeValuedBitvector<X>;

    fn uext(self, new_bound: X) -> Self::Output {
        let old_width = self.bound().width();
        let new_width = new_bound.width();

        // set width first
        let mut zeros = self.zeros.uext(new_bound);
        let ones = self.ones.uext(new_bound);

        // if extending, extend zeros by 1s
        if new_width > old_width {
            let num_set_bits = new_width - old_width;
            if let Some(hi) = new_bound.highest_bit() {
                let lo = hi.saturating_sub(num_set_bits - 1);
                zeros.set_bits(lo, hi, true);
            }
        }

        ThreeValuedBitvector::from_zeros_ones(zeros, ones)
    }

    fn sext(self, new_bound: X) -> Self::Output {
        // we need to extend both by their highest bit
        if self.bound().width() == 0 {
            // no zeros nor ones, handle specially by returning zero
            // (with all zeros filled)
            return ThreeValuedBitvector::new(0, new_bound);
        }

        let zeros = self.zeros.sext(new_bound);
        let ones = self.ones.sext(new_bound);

        ThreeValuedBitvector::from_zeros_ones(zeros, ones)
    }
}

impl<const W: u32, const X: u32> Ext<X> for ThreeValuedBitvector<CBound<W>> {
    type Output = ThreeValuedBitvector<CBound<X>>;

    fn uext(self) -> Self::Output {
        BExt::uext(self, CBound::<X>)
    }

    fn sext(self) -> Self::Output {
        BExt::sext(self, CBound::<X>)
    }
}
