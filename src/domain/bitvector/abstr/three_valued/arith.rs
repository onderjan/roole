use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, ExtendedBitvectorDomain},
        bound::compute_u64_mask,
        concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector},
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

    fn udiv_wrapping_or_full(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        let (dividend_umin, dividend_umax) = (self.umin(), self.umax());
        let (mut divisor_umin, divisor_umax) = (rhs.umin(), rhs.umax());

        let can_be_div_by_zero = divisor_umin.is_zero();

        if can_be_div_by_zero {
            if divisor_umax.is_zero() {
                // always division by zero
                // this function produces full bitvector on division by zero
                return Self::new_full(bound);
            }

            // compute division from 1 up
            divisor_umin = UnsignedBitvector::one(bound);
        }

        let min_division_result = dividend_umin / divisor_umax;
        let max_division_result = dividend_umax / divisor_umin;

        let mut quotient = convert_uarith(
            min_division_result.to_u64(),
            max_division_result.to_u64(),
            bound,
        );

        if can_be_div_by_zero {
            // this function produces full bitvector on division by zero
            quotient.apply_join(&Self::new_full(bound))
        }

        quotient
    }

    fn sdiv_wrapping_or_full(self, rhs: Self) -> Self {
        todo!("sdiv")
    }

    /*fn sdiv_wrapping_or_zero(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());

        let result = compute_sdivrem(self, rhs, |a, b| (a / b).result);
        zero_div_to_zero(rhs, result)
    }*/

    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        let (dividend_umin, dividend_umax) = (self.umin(), self.umax());
        let (mut divisor_umin, divisor_umax) = (rhs.umin(), rhs.umax());

        let can_be_div_by_zero = divisor_umin.is_zero();

        if can_be_div_by_zero {
            if divisor_umax.is_zero() {
                // always division by zero
                // this function produces dividend on division by zero
                return self;
            }

            // compute division from 1 up
            divisor_umin = UnsignedBitvector::one(bound);
        }

        let min_division_result = dividend_umin / divisor_umax;
        let max_division_result = dividend_umax / divisor_umin;

        let mut remainder = if min_division_result == max_division_result {
            // only one division result, compute remainder
            let min_result = (dividend_umin % divisor_umax).to_u64();
            let max_result = (dividend_umax % divisor_umin).to_u64();
            convert_uarith(min_result, max_result, bound)
        } else {
            // more than one division result, return fully unknown remainder
            // TODO: this can be restricted to be lesser than divisor
            Self::new_unknown(bound)
        };

        if can_be_div_by_zero {
            // this function produces dividend on division by zero
            remainder.apply_join(&self)
        }

        remainder
    }

    fn srem_wrapping_or_dividend(self, rhs: Self) -> Self {
        todo!("srem")
    }

    /*fn srem_wrapping_or_zero(self, rhs: Self) -> Self {
        assert_eq!(self.bound(), rhs.bound());
        let bound = self.bound();

        let sdiv_result = self.sdiv_wrapping_or_zero(rhs);
        if sdiv_result.concrete_value().is_none() {
            // sdiv is not a concrete value, make fully unknown
            let result = Self::new_unknown(bound);
            return zero_div_to_zero(rhs, result);
        }

        let result = compute_sdivrem(self, rhs, |a, b| a % b);
        zero_div_to_zero(rhs, result)
    }*/
}

/*fn zero_div_to_zero<B: BitvectorBound>(
    divisor: ThreeValuedBitvector<B>,
    mut result: ThreeValuedBitvector<B>,
) -> ThreeValuedBitvector<B> {
    // in SMT-LIB, division by zero produces zero
    let bound = divisor.bound();
    let zero = ConcreteBitvector::zero(bound);
    let can_be_corner = divisor.contains_concrete(&zero);
    let must_be_corner = divisor.concrete_value().map(|v| v == zero).unwrap_or(false);
    if must_be_corner {
        result = ThreeValuedBitvector::new(0, bound)
    } else if can_be_corner {
        result = result.join(&ThreeValuedBitvector::new(0, bound))
    };
    result
}*/

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

fn convert_uarith<B: BitvectorBound>(min: u64, max: u64, bound: B) -> ThreeValuedBitvector<B> {
    // make highest different bit and all after it unknown
    let different = min ^ max;
    if different == 0 {
        // both are the same
        return ThreeValuedBitvector::new(min, bound);
    }

    let highest_different_bit_pos = different.ilog2();
    let unknown_mask = compute_u64_mask(highest_different_bit_pos + 1);
    ThreeValuedBitvector::new_value_unknown(
        ConcreteBitvector::new(min, bound),
        ConcreteBitvector::new(unknown_mask, bound),
    )
}

/*

fn compute_sdivrem<B: BitvectorBound>(
    dividend: ThreeValuedBitvector<B>,
    divisor: ThreeValuedBitvector<B>,
    op_fn: fn(SignedBitvector<B>, SignedBitvector<B>) -> SignedBitvector<B>,
) -> ThreeValuedBitvector<B> {
    let bound = dividend.bound();
    let width = bound.width();

    if width == 0 {
        // prevent problems
        return dividend;
    }

    let const_one = if width > 1 {
        SignedBitvector::new(1, bound)
    } else {
        SignedBitvector::new(-1, bound)
    };

    let mut zeros = 0u64;
    let mut ones = 0u64;

    let divisor_min = divisor.smin();
    let divisor_max = divisor.smax();
    // handle positive, 0, -1, negative below -1 separately
    if divisor_max.to_i64() > 0 {
        // handle positive divisor
        let divisor_min = if divisor_min.to_i64() > 1 {
            divisor_min
        } else {
            const_one
        };

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            divisor_min,
            divisor_max,
            op_fn,
        );
    }

    if divisor_min.to_i64() <= 0 && divisor_max.to_i64() >= 0 {
        // 0 divisor, causes division by zero, handle separately

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            SignedBitvector::new(0, bound),
            SignedBitvector::new(0, bound),
            op_fn,
        );
    }

    if divisor_min.to_i64() <= -1 && divisor_max.to_i64() >= -1 {
        // -1 divisor, causes overflow when the dividend is the most negative value, handle separately

        let minus_one = ConcreteBitvector::bit_mask(bound).as_signed();

        let mut dividend_min = dividend.smin();
        let dividend_max = dividend.smax();

        if dividend_min == ConcreteBitvector::sign_bit_mask(bound).as_signed() {
            // overflow
            apply_signed_op(
                &mut zeros,
                &mut ones,
                dividend_min,
                dividend_min,
                minus_one,
                minus_one,
                op_fn,
            );
            if dividend_min != dividend_max {
                dividend_min = dividend_min + const_one;
            }
        }

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend_min,
            dividend_max,
            minus_one,
            minus_one,
            op_fn,
        );
    }

    if divisor_min.to_i64() < -1 {
        // handle negative divisor
        let divisor_max = if divisor_max.to_i64() < -1 {
            divisor_max
        } else {
            SignedBitvector::new(-2, bound)
        };

        apply_signed_op(
            &mut zeros,
            &mut ones,
            dividend.smin(),
            dividend.smax(),
            divisor_min,
            divisor_max,
            op_fn,
        );
    }

    ThreeValuedBitvector::from_zeros_ones(
        ConcreteBitvector::new(zeros, bound),
        ConcreteBitvector::new(ones, bound),
    )
}

fn apply_signed_op<B: BitvectorBound>(
    zeros: &mut u64,
    ones: &mut u64,
    a_min: SignedBitvector<B>,
    a_max: SignedBitvector<B>,
    b_min: SignedBitvector<B>,
    b_max: SignedBitvector<B>,
    op_fn: fn(SignedBitvector<B>, SignedBitvector<B>) -> SignedBitvector<B>,
) {
    let bound = a_min.cast_bitvector().bound();
    // apply all configurations
    // cast to unsigned u64 afterwards
    let x = op_fn(a_min, b_min).cast_bitvector().as_unsigned().to_u64();
    let y = op_fn(a_min, b_max).cast_bitvector().as_unsigned().to_u64();
    let z = op_fn(a_max, b_min).cast_bitvector().as_unsigned().to_u64();
    let w = op_fn(a_max, b_max).cast_bitvector().as_unsigned().to_u64();

    // find the highest different bit
    let found_zeros = (!x | !y | !z | !w) & bound.mask();
    let found_ones = x | y | z | w;
    let different = found_zeros & found_ones;

    // apply them
    *zeros |= found_zeros;
    *ones |= found_ones;

    if different == 0 {
        // all are the same
        return;
    }

    // also take care of the lower bits

    let highest_different_bit_pos = different.ilog2();
    let unknown_mask = compute_u64_mask(highest_different_bit_pos + 1);

    *zeros |= unknown_mask;
    *ones |= unknown_mask;
}*/
