use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, ExtendedBitvectorDomain},
        bound::compute_u64_mask,
        concr::{ConcreteBitvector, UnsignedBitvector},
        interval::{SignlessInterval, UnsignedInterval},
    },
    traits::{Join, forward::HwArith},
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> HwArith for ThreeValuedBitvector<B> {
    fn arith_neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::new(0, self.bound()), self)
    }
    fn add(self, rhs: Self) -> Self {
        // return early if one of arguments is zero
        let is_zero = |val: ConcreteBitvector<B>| val.is_zero();
        if self.concrete_value().is_some_and(is_zero) {
            return rhs;
        }
        if rhs.concrete_value().is_some_and(is_zero) {
            return self;
        }

        minmax_compute(self, rhs, |lhs, rhs, k| {
            addsub_zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umin(),
                rhs.umax(),
                k,
                |lhs, rhs| lhs.overflowing_add(rhs),
            )
        })
    }
    fn sub(self, rhs: Self) -> Self {
        // return early if rhs is zero
        if rhs.concrete_value().is_some_and(|val| val.is_zero()) {
            return self;
        }

        minmax_compute(self, rhs, |lhs, rhs, k| {
            // swap rhs min and max as it is applied in negative
            addsub_zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umax(),
                rhs.umin(),
                k,
                |lhs, rhs| lhs.overflowing_sub(rhs),
            )
        })
    }
    fn mul(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let is_zero = |val: ConcreteBitvector<B>| val.is_zero();
        let is_one = |val: ConcreteBitvector<B>| val.is_one();
        // return zero if one is zero, return the other argument if an argument is one
        if self.concrete_value().is_some_and(is_zero) || rhs.concrete_value().is_some_and(is_one) {
            return self;
        }
        if rhs.concrete_value().is_some_and(is_zero) || self.concrete_value().is_some_and(is_one) {
            return rhs;
        }

        // use the minmax algorithm for now
        minmax_compute(self, rhs, |lhs, rhs, k| {
            // prepare a mask that selects interval [0, k]
            let mod_mask = compute_u64_mask(k + 1);

            // convert all to u128 so there is no overflow
            let left_min = (lhs.umin().to_u64() & mod_mask) as u128;
            let right_min = (rhs.umin().to_u64() & mod_mask) as u128;
            let left_max = (lhs.umax().to_u64() & mod_mask) as u128;
            let right_max = (rhs.umax().to_u64() & mod_mask) as u128;

            let zeta_k_min = ((left_min * right_min) >> k) as u64;
            let zeta_k_max = ((left_max * right_max) >> k) as u64;
            (zeta_k_min, zeta_k_max)
        })
    }

    fn udiv_wrapping_or_all_ones(self, rhs: Self) -> Self {
        let dividend = self.unsigned_interval();
        let divisor = rhs.unsigned_interval();
        Self::from_unsigned_interval(dividend.udiv_wrapping_or_full(divisor))
    }

    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self {
        let dividend = self.unsigned_interval();
        let divisor = rhs.unsigned_interval();
        Self::from_unsigned_interval(dividend.urem_wrapping_or_full(divisor))
    }

    fn sdiv_wrapping_by_quadrants(self, rhs: Self) -> Self {
        handle_by_quadrants(self, rhs, true)
    }

    fn srem_wrapping_by_quadrants(self, rhs: Self) -> Self {
        handle_by_quadrants(self, rhs, false)
    }
}

fn handle_by_quadrants<B: BitvectorBound>(
    dividend: ThreeValuedBitvector<B>,
    divisor: ThreeValuedBitvector<B>,
    is_division: bool,
) -> ThreeValuedBitvector<B> {
    let bound = dividend.bound();
    assert_eq!(bound, divisor.bound());

    if bound.width() == 0 {
        // return early
        return dividend;
    }

    // split into four quadrants

    let (dividend_neg_half, dividend_zpos_half) = dividend.signed_interval().into_signless_halves();
    let (divisor_neg_half, divisor_zpos_half) = divisor.signed_interval().into_signless_halves();

    let dividend_neg_half = dividend_neg_half.map(SignlessInterval::into_unsigned);
    let dividend_zpos_half = dividend_zpos_half.map(SignlessInterval::into_unsigned);

    let divisor_neg_half = divisor_neg_half.map(SignlessInterval::into_unsigned);
    let divisor_zpos_half = divisor_zpos_half.map(SignlessInterval::into_unsigned);

    // handle each quadrant separately

    let mut result: Option<ThreeValuedBitvector<B>> = None;

    let mut combine_result = |quadrant_result| {
        let quadrant_result = ThreeValuedBitvector::from_unsigned_interval(quadrant_result);
        if let Some(result) = result.as_mut() {
            result.apply_join(&quadrant_result);
        } else {
            result = Some(quadrant_result);
        }
    };

    let op: fn(UnsignedInterval<B>, UnsignedInterval<B>) -> UnsignedInterval<B> = if is_division {
        UnsignedInterval::udiv_wrapping_or_full
    } else {
        UnsignedInterval::urem_wrapping_or_full
    };

    if let (Some(a), Some(b)) = (dividend_zpos_half, divisor_zpos_half) {
        // perform unsigned operation normally
        combine_result((op)(a, b))
    }
    if let (Some(a), Some(b)) = (dividend_neg_half, divisor_zpos_half) {
        // (-) / (+)
        // negate dividend, perform operation, then negate result
        combine_result((op)(a.arith_neg(), b).arith_neg())
    }
    if let (Some(a), Some(b)) = (dividend_zpos_half, divisor_neg_half) {
        // (+) / (-)

        if is_division {
            // negate divisor, perform division, then negate result
            combine_result((op)(a, b.arith_neg()).arith_neg());
        } else {
            // negate divisor, perform remainder, but DO NOT negate the result
            combine_result((op)(a, b.arith_neg()));
        }
    }
    if let (Some(a), Some(b)) = (dividend_neg_half, divisor_neg_half) {
        // (-) / (-)
        if is_division {
            // negate both, perform division
            combine_result((op)(a.arith_neg(), b.arith_neg()))
        } else {
            // negate both, perform division, negate the result
            combine_result((op)(a.arith_neg(), b.arith_neg()).arith_neg())
        }
    }

    result.expect("Signed division/remainder must have at least one quadrant")
}

fn minmax_compute<B: BitvectorBound>(
    lhs: ThreeValuedBitvector<B>,
    rhs: ThreeValuedBitvector<B>,
    zeta_k_fn: fn(ThreeValuedBitvector<B>, ThreeValuedBitvector<B>, u32) -> (u64, u64),
) -> ThreeValuedBitvector<B> {
    let bound = lhs.bound();
    let width = bound.width();
    // from previous paper

    // start with no possibilites
    let mut ones = 0u64;
    let mut zeros = 0u64;

    // iterate over output bits
    for k in 0..width {
        // compute h_k extremes
        let (zeta_k_min, zeta_k_max) = zeta_k_fn(lhs, rhs, k);

        // see if minimum and maximum differs
        if zeta_k_min != zeta_k_max {
            // set result bit unknown
            zeros |= 1 << k;
            ones |= 1 << k;
        } else {
            // set value of bit k, converted to ones-zeros encoding
            zeros |= (!zeta_k_min & 1) << k;
            ones |= (zeta_k_min & 1) << k;
        }
    }
    ThreeValuedBitvector::from_zeros_ones(
        ConcreteBitvector::new(zeros, bound),
        ConcreteBitvector::new(ones, bound),
    )
}

fn addsub_zeta_k_fn<B: BitvectorBound>(
    left_min: UnsignedBitvector<B>,
    left_max: UnsignedBitvector<B>,
    right_min: UnsignedBitvector<B>,
    right_max: UnsignedBitvector<B>,
    k: u32,
    func: fn(u64, u64) -> (u64, bool),
) -> (u64, u64) {
    // prepare a mask that selects interval [0, k]
    let mod_mask = compute_u64_mask(k + 1);

    let left_min = left_min.to_u64() & mod_mask;
    let left_max = left_max.to_u64() & mod_mask;
    let right_min = right_min.to_u64() & mod_mask;
    let right_max = right_max.to_u64() & mod_mask;

    // shift right, using the overflow as well
    let zeta_k_min = shr_overflowing(func(left_min, right_min), k);
    let zeta_k_max = shr_overflowing(func(left_max, right_max), k);

    (zeta_k_min, zeta_k_max)
}

fn shr_overflowing(overflowing_result: (u64, bool), k: u32) -> u64 {
    let mut result = overflowing_result.0 >> k;
    if overflowing_result.1 && k > 0 {
        let overflow_pos = u64::BITS - k;
        result |= 1u64 << overflow_pos;
    }
    result
}
