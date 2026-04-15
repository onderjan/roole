use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, ExtendedBitvectorDomain},
        concr::ConcreteBitvector,
    },
    traits::forward::{Bitwise, HwShift},
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> HwShift for ThreeValuedBitvector<B> {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        assert_eq!(self.bound(), amount.bound());

        // shifting left logically, we need to shift in zeros from right
        let zeros_shift_fn = |value: ConcreteBitvector<B>, amount: ConcreteBitvector<B>| {
            let bit_mask = ConcreteBitvector::new_all_ones(value.bound());
            let shifted_mask = bit_mask.logic_shl(amount.clone());
            Bitwise::bit_or(value.logic_shl(amount), shifted_mask.bit_not())
        };
        let ones_shift_fn =
            |value: ConcreteBitvector<B>, amount: ConcreteBitvector<B>| value.logic_shl(amount);

        shift(
            &self,
            &amount,
            zeros_shift_fn,
            ones_shift_fn,
            &Self::new(0, self.bound()),
        )
    }

    fn logic_shr(self, amount: Self) -> Self {
        assert_eq!(self.bound(), amount.bound());

        // shifting right logically, we need to shift in zeros from left
        let zeros_shift_fn = |value: ConcreteBitvector<B>, amount: ConcreteBitvector<B>| {
            let bit_mask = ConcreteBitvector::new_all_ones(value.bound());
            let shifted_mask = bit_mask.logic_shr(amount.clone());
            Bitwise::bit_or(value.logic_shr(amount), shifted_mask.bit_not())
        };
        let ones_shift_fn =
            |value: ConcreteBitvector<B>, amount: ConcreteBitvector<B>| value.logic_shr(amount);

        shift(
            &self,
            &amount,
            zeros_shift_fn,
            ones_shift_fn,
            &Self::new(0, self.bound()),
        )
    }

    fn arith_shr(self, amount: Self) -> Self {
        assert_eq!(self.bound(), amount.bound());
        let bound = self.bound();
        let bit_mask = ConcreteBitvector::new_all_ones(bound);

        // shifting right arithmetically, we need to shift in the sign bit from left
        let sra_shift_fn = |value: ConcreteBitvector<B>, amount: ConcreteBitvector<B>| {
            if value.is_sign_bit_set() {
                let bit_mask = ConcreteBitvector::new_all_ones(value.bound());
                let shifted_mask = bit_mask.logic_shr(amount.clone());
                Bitwise::bit_or(value.logic_shr(amount), shifted_mask.bit_not())
            } else {
                value.logic_shr(amount)
            }
        };

        // the overflow value is determined by sign bit
        let overflow_zeros = if self.is_zeros_sign_bit_set() {
            bit_mask.clone()
        } else {
            ConcreteBitvector::new(0, self.bound())
        };

        let overflow_ones = if self.is_ones_sign_bit_set() {
            bit_mask
        } else {
            ConcreteBitvector::new(0, self.bound())
        };
        let overflow_value = Self::from_zeros_ones(overflow_zeros, overflow_ones);

        shift(&self, &amount, sra_shift_fn, sra_shift_fn, &overflow_value)
    }
}

fn shift<B: BitvectorBound>(
    value: &ThreeValuedBitvector<B>,
    amount: &ThreeValuedBitvector<B>,
    zeros_shift_fn: impl Fn(ConcreteBitvector<B>, ConcreteBitvector<B>) -> ConcreteBitvector<B>,
    ones_shift_fn: impl Fn(ConcreteBitvector<B>, ConcreteBitvector<B>) -> ConcreteBitvector<B>,
    overflow_value: &ThreeValuedBitvector<B>,
) -> ThreeValuedBitvector<B> {
    assert_eq!(value.bound(), amount.bound());
    let bound = value.bound();
    let width = bound.width();
    if width == 0 {
        // avoid problems with zero-bound bitvectors
        return value.clone();
    }

    let mut zeros = ConcreteBitvector::new(0, bound);
    let mut ones = ConcreteBitvector::new(0, bound);

    let umin = amount.umin().cast_bitvector().try_to_u32();
    let umax = amount.umax().cast_bitvector().try_to_u32();

    // the shift amount is also three-valued, which poses problems
    // first, if it can be shifted by L or larger value, join by overflow value
    let shift_can_overflow = umax.is_none_or(|umax| umax >= width);
    if shift_can_overflow {
        zeros = zeros.bit_or(overflow_value.zeros.clone());
        ones = ones.bit_or(overflow_value.ones.clone());
    }

    let Some(umin) = umin else {
        // only the overflow value is possible
        return ThreeValuedBitvector::from_zeros_ones(zeros, ones);
    };

    let max_nonoverflowing = width - 1;
    if umin > max_nonoverflowing {
        // only the overflow value is possible
        return ThreeValuedBitvector::from_zeros_ones(zeros, ones);
    }

    // we need to only consider the amounts smaller than width now
    let min_shift = umin;
    let max_shift = umax.unwrap_or(max_nonoverflowing).min(max_nonoverflowing);
    // join by the other shifts iteratively
    for i in min_shift..=max_shift {
        let bi = ConcreteBitvector::new(i.into(), bound);
        if amount.contains_concrete(&bi) {
            let shifted_zeros = zeros_shift_fn(value.zeros.clone(), bi.clone());
            let shifted_ones = ones_shift_fn(value.ones.clone(), bi);
            zeros = zeros.bit_or(shifted_zeros);
            ones = ones.bit_or(shifted_ones);
        }
    }
    ThreeValuedBitvector::from_zeros_ones(zeros, ones)
}
