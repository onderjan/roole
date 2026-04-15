use std::fmt::Debug;

use crate::domain::traits::forward::HwArith;

use super::{
    super::{
        BitvectorBound,
        concr::{ConcreteBitvector, UnsignedBitvector},
    },
    SignlessInterval,
};

/// An unsigned interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct UnsignedInterval<B: BitvectorBound> {
    min: UnsignedBitvector<B>,
    max: UnsignedBitvector<B>,
}

impl<B: BitvectorBound> UnsignedInterval<B> {
    pub fn new(min: UnsignedBitvector<B>, max: UnsignedBitvector<B>) -> Self {
        // comparison will panic on different bound values
        assert!(min <= max);
        Self { min, max }
    }

    pub fn try_new(min: UnsignedBitvector<B>, max: UnsignedBitvector<B>) -> Result<Self, ()> {
        if min <= max {
            Ok(Self { min, max })
        } else {
            Err(())
        }
    }

    // the canonical full interval is from umin (zero) to umax (full mask)
    pub fn new_full(bound: B) -> Self {
        Self {
            min: ConcreteBitvector::new_zero(bound).into_unsigned(),
            max: ConcreteBitvector::new_all_ones(bound).into_unsigned(),
        }
    }

    pub fn bound(&self) -> B {
        // the bound must be the same for min and max
        self.min.bound()
    }

    pub fn min(&self) -> &UnsignedBitvector<B> {
        &self.min
    }
    pub fn max(&self) -> &UnsignedBitvector<B> {
        &self.max
    }

    pub fn into_min_max(self) -> (UnsignedBitvector<B>, UnsignedBitvector<B>) {
        (self.min, self.max)
    }

    pub fn ext<X: BitvectorBound>(self, new_bound: X) -> UnsignedInterval<X> {
        if self.min == self.max {
            // clearly, we can extend
            let ext_value = self.min.ext(new_bound);
            return UnsignedInterval {
                min: ext_value.clone(),
                max: ext_value,
            };
        }

        // if we narrow the interval and disregarded a bound, saturate
        let mut ext_min: UnsignedBitvector<X> = self.min.clone().ext(new_bound);
        let mut ext_max: UnsignedBitvector<X> = self.max.clone().ext(new_bound);

        let old_bound = self.bound();
        let min_diff: UnsignedBitvector<B> = self.min - ext_min.clone().ext(old_bound);
        let max_diff: UnsignedBitvector<B> = self.max - ext_max.clone().ext(old_bound);

        if min_diff != max_diff {
            // we disregarded a bound, saturate
            ext_min = ConcreteBitvector::new_zero(new_bound).into_unsigned();
            ext_max = ConcreteBitvector::new_all_ones(new_bound).into_unsigned();
        }
        UnsignedInterval {
            min: ext_min,
            max: ext_max,
        }
    }

    pub fn try_into_signless(self) -> Option<SignlessInterval<B>> {
        let min = self.min.cast_bitvector();
        let max = self.max.cast_bitvector();

        if min.is_sign_bit_set() == max.is_sign_bit_set() {
            Some(SignlessInterval::new(min, max))
        } else {
            None
        }
    }

    pub fn arith_neg(self) -> Self {
        let bound = self.bound();
        let new_max = self.min.cast_bitvector().arith_neg().into_unsigned();
        let new_min = self.max.cast_bitvector().arith_neg().into_unsigned();
        // it is possible this will no longer be unsigned, make full in that case
        if let Ok(result) = Self::try_new(new_min, new_max) {
            result
        } else {
            Self::new_full(bound)
        }
    }

    /*pub fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
        let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

        let x_diff_mask = mask_from_leading_one(x_p ^ x_q);
        let y_diff_mask = mask_from_leading_one(y_p ^ y_q);
        let diff_mask = x_diff_mask | y_diff_mask;

        let min = x_p & y_p & !mask_from_leading_one(!x_p & !y_p & diff_mask);
        let max = {
            let selection_x = mask_from_leading_one(x_q & !y_q & x_diff_mask);
            let selection_y = mask_from_leading_one(y_q & !x_q & y_diff_mask);

            let result_q = x_q & y_q;
            let result_x = selection_x & (y_q & !x_q);
            let result_y = selection_y & (x_q & !y_q);
            result_q | result_x.max(result_y)
        };

        Self::new(
            ConcreteBitvector::new(min, bound).into_unsigned(),
            ConcreteBitvector::new(max, bound).into_unsigned(),
        )
    }

    pub fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
        let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

        let x_diff_mask = mask_from_leading_one(x_p ^ x_q);
        let y_diff_mask = mask_from_leading_one(y_p ^ y_q);
        let diff_mask = x_diff_mask | y_diff_mask;

        let min = {
            let candidates_x = y_p & !x_p & x_diff_mask;
            let candidates_y = x_p & !y_p & y_diff_mask;

            if candidates_x >= candidates_y {
                let selection_x = mask_from_leading_one(candidates_x);
                (x_p & !selection_x) | y_p
            } else {
                let selection_y = mask_from_leading_one(candidates_y);
                (y_p & !selection_y) | x_p
            }
        };
        let max = x_q | y_q | mask_from_leading_one(x_q & y_q & diff_mask);

        Self::new(
            ConcreteBitvector::new(min, bound).into_unsigned(),
            ConcreteBitvector::new(max, bound).into_unsigned(),
        )
    }

    pub fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        // An improvement of the Hacker's Delight algorithm, giving O(1) computation
        // if Count Leading Zeros (clz) is implemented.

        let (x_p, x_q) = (self.min.to_u64(), self.max.to_u64());
        let (y_p, y_q) = (rhs.min.to_u64(), rhs.max.to_u64());

        let diff_mask = mask_from_leading_one((x_p ^ x_q) | (y_p ^ y_q));

        let min = {
            let y_q_mask = mask_from_leading_one(!x_p & y_q & diff_mask);
            let x_q_mask = mask_from_leading_one(!y_p & x_q & diff_mask);
            (x_p & !y_q & !y_q_mask) | (y_p & !x_q & !x_q_mask)
        };

        let max = {
            let neither_p_mask = mask_from_leading_one(!x_p & !y_p & diff_mask);
            let both_q_mask = mask_from_leading_one(y_q & x_q & diff_mask);
            (!x_p | !y_p | neither_p_mask) & (x_q | y_q | both_q_mask)
        };

        // we need to mask max
        Self::new(
            ConcreteBitvector::from_masked_u64(min, bound).into_unsigned(),
            ConcreteBitvector::from_masked_u64(max, bound).into_unsigned(),
        )
    }*/

    pub fn udiv_wrapping_or_full(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (dividend_umin, dividend_umax) = (self.min, self.max);
        let (mut divisor_umin, divisor_umax) = (rhs.min, rhs.max);

        let can_be_div_by_zero = divisor_umin.is_zero();

        if can_be_div_by_zero {
            if divisor_umax.is_zero() {
                // always division by zero
                // this function produces all-ones bitvector on division by zero
                let all_ones = ConcreteBitvector::new_all_ones(bound).into_unsigned();
                return Self::new(all_ones.clone(), all_ones);
            }

            // compute division from 1 up
            divisor_umin = UnsignedBitvector::new_one(bound);
        }

        let min_division_result = dividend_umin.div_wrapping_or_full(divisor_umax);
        let mut max_division_result = dividend_umax.div_wrapping_or_full(divisor_umin);

        if can_be_div_by_zero {
            // this function produces all-ones bitvector on division by zero
            max_division_result = ConcreteBitvector::new_all_ones(bound).into_unsigned();
        }

        UnsignedInterval::new(min_division_result, max_division_result)
    }

    pub fn urem_wrapping_or_full(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        let (dividend_umin, dividend_umax) = (self.min.clone(), self.max.clone());
        let (mut divisor_umin, divisor_umax) = (rhs.min, rhs.max);

        let can_be_div_by_zero = divisor_umin.is_zero();

        if can_be_div_by_zero {
            if divisor_umax.is_zero() {
                // always division by zero
                // this function produces dividend on division by zero
                return self;
            }

            // compute division from 1 up
            divisor_umin = UnsignedBitvector::new_one(bound);
        }

        let min_division_result = dividend_umin
            .clone()
            .div_wrapping_or_full(divisor_umax.clone());
        let max_division_result = dividend_umax
            .clone()
            .div_wrapping_or_full(divisor_umin.clone());

        let mut remainder = if min_division_result == max_division_result {
            // only one division result, compute remainder
            let min_remainder = dividend_umin.clone().rem_wrapping_or_dividend(divisor_umax);
            let max_remainder = dividend_umax.clone().rem_wrapping_or_dividend(divisor_umin);
            Self::new(min_remainder, max_remainder)
        } else {
            // more than one division result, return fully unknown remainder
            // TODO: this could be restricted to be lesser than divisor
            Self::new_full(bound)
        };

        if can_be_div_by_zero {
            // this function produces dividend on division by zero
            // make sure the minimum is at most dividend minimum and maximum is at least dividend maximum
            if remainder.min > dividend_umin {
                remainder.min = dividend_umin;
            }
            if remainder.max < dividend_umax {
                remainder.max = dividend_umax;
            }
        }

        remainder
    }

    #[allow(dead_code)]
    pub fn contains_value(&self, value: &UnsignedBitvector<B>) -> bool {
        &self.min <= value && value <= &self.max
    }
}

fn mask_from_leading_one(x: u64) -> u64 {
    let diff_clz = x.leading_zeros();
    u64::MAX.checked_shr(diff_clz).unwrap_or(0)
}

impl<B: BitvectorBound> Debug for UnsignedInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}
