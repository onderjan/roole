use crate::domain::{
    bitvector::{
        BitvectorBound, RBound,
        abstr::{BitvectorDomain, ExtendedBitvectorDomain},
        concr::{ConcreteBitvector, UnsignedBitvector},
        interval::{SignlessInterval, UnsignedInterval},
    },
    traits::{
        Join,
        forward::{BExt, HwArith, HwShift},
    },
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> HwArith for ThreeValuedBitvector<B> {
    fn arith_neg(self) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0
        HwArith::sub(Self::new_zero(self.bound()), self)
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
            zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umin(),
                rhs.umax(),
                k,
                |w| w + 1,
                |lhs, rhs| lhs.add(rhs),
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
            zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umax(),
                rhs.umin(),
                k,
                |w| w + 1,
                |lhs, rhs| lhs.sub(rhs),
            )
        })
    }
    fn mul(self, rhs: Self) -> Self {
        minmax_compute(self, rhs, |lhs, rhs, k| {
            zeta_k_fn(
                lhs.umin(),
                lhs.umax(),
                rhs.umin(),
                rhs.umax(),
                k,
                |w| w * 2,
                |lhs, rhs| lhs.mul(rhs),
            )
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
        handle_by_quadrants(self, rhs, |a, a_is_neg, b, b_is_neg| {
            match (a_is_neg, b_is_neg) {
                (false, false) => a.udiv_wrapping_or_full(b),
                (true, false) => {
                    // negate dividend, compute, then negate result
                    a.arith_neg().udiv_wrapping_or_full(b).arith_neg()
                }
                (false, true) => {
                    // negate divisor, compute, then negate result
                    a.udiv_wrapping_or_full(b.arith_neg()).arith_neg()
                }
                (true, true) => {
                    // negate both, compute
                    a.arith_neg().udiv_wrapping_or_full(b.arith_neg())
                }
            }
        })
    }

    fn srem_wrapping_by_quadrants(self, rhs: Self) -> Self {
        handle_by_quadrants(self, rhs, |a, a_is_neg, b, b_is_neg| {
            match (a_is_neg, b_is_neg) {
                (false, false) => a.urem_wrapping_or_full(b),
                (true, false) => {
                    // negate dividend, compute, then negate result
                    a.arith_neg().urem_wrapping_or_full(b).arith_neg()
                }
                (false, true) => {
                    // negate divisor, compute, but DO NOT negate the result
                    a.urem_wrapping_or_full(b.arith_neg())
                }
                (true, true) => {
                    // negate both, compute, negate the result
                    a.arith_neg()
                        .urem_wrapping_or_full(b.arith_neg())
                        .arith_neg()
                }
            }
        })
    }

    fn smod_wrapping_by_quadrants(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // handle only if both values are concrete
        // TODO: more precise handling
        let (Some(lhs), Some(rhs)) = (self.concrete_value(), rhs.concrete_value()) else {
            return Self::new_unknown(bound);
        };

        Self::from_concrete_value(lhs.smod_wrapping_by_quadrants(rhs))
    }
}

type QuadrantHandler<B> = fn(
    lhs: UnsignedInterval<B>,
    lhs_neg: bool,
    rhs: UnsignedInterval<B>,
    rhs_neg: bool,
) -> UnsignedInterval<B>;

fn handle_by_quadrants<B: BitvectorBound>(
    dividend: ThreeValuedBitvector<B>,
    divisor: ThreeValuedBitvector<B>,
    quadrant_handler: QuadrantHandler<B>,
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

    /*let op: fn(UnsignedInterval<B>, UnsignedInterval<B>) -> UnsignedInterval<B> = if is_division {
        UnsignedInterval::udiv_wrapping_or_full
    } else {
        UnsignedInterval::urem_wrapping_or_full
    };*/

    if let (Some(a), Some(b)) = (dividend_zpos_half.clone(), divisor_zpos_half.clone()) {
        // (+) / (+)

        combine_result(quadrant_handler(a, false, b, false));
        // perform unsigned operation normally
        //combine_result((op)(a, b))
    }
    if let (Some(a), Some(b)) = (dividend_neg_half.clone(), divisor_zpos_half) {
        // (-) / (+)
        combine_result(quadrant_handler(a, true, b, false));

        // negate dividend, perform operation, then negate result
        //combine_result((op)(a.arith_neg(), b).arith_neg())
    }
    if let (Some(a), Some(b)) = (dividend_zpos_half, divisor_neg_half.clone()) {
        // (+) / (-)
        combine_result(quadrant_handler(a, false, b, true));

        /*if is_division {
            // negate divisor, perform division, then negate result
            combine_result((op)(a, b.arith_neg()).arith_neg());
        } else {
            // negate divisor, perform remainder, but DO NOT negate the result
            combine_result((op)(a, b.arith_neg()));
        }*/
    }
    if let (Some(a), Some(b)) = (dividend_neg_half, divisor_neg_half) {
        // (-) / (-)
        combine_result(quadrant_handler(a, true, b, true));
        /*(if is_division {
            // negate both, perform division
            combine_result((op)(a.arith_neg(), b.arith_neg()))
        } else {
            // negate both, perform division, negate the result
            combine_result((op)(a.arith_neg(), b.arith_neg()).arith_neg())
        }*/
    }

    result.expect("Signed division/remainder must have at least one quadrant")
}

type ZetaFn<B> = fn(
    &ThreeValuedBitvector<B>,
    &ThreeValuedBitvector<B>,
    u32,
) -> (ConcreteBitvector<RBound>, ConcreteBitvector<RBound>);

fn minmax_compute<B: BitvectorBound>(
    lhs: ThreeValuedBitvector<B>,
    rhs: ThreeValuedBitvector<B>,
    zeta_k_fn: ZetaFn<B>,
) -> ThreeValuedBitvector<B> {
    let bound = lhs.bound();
    let width = bound.width();
    // from previous paper

    // start with no possibilites
    let mut ones = ConcreteBitvector::new_zero(bound);
    let mut zeros = ConcreteBitvector::new_zero(bound);

    // iterate over output bits
    for k in 0..width {
        // compute h_k extremes
        let (zeta_k_min, zeta_k_max) = zeta_k_fn(&lhs, &rhs, k);

        // see if minimum and maximum differs
        if zeta_k_min != zeta_k_max {
            // set result bit unknown
            zeros.set_bit(k, true);
            ones.set_bit(k, true);
        } else {
            // set value of bit k, converted to ones-zeros encoding
            let value = zeta_k_min.is_bit_set(0);
            if value {
                ones.set_bit(k, true);
            } else {
                zeros.set_bit(k, true);
            }
        }
    }
    ThreeValuedBitvector::from_zeros_ones(zeros, ones)
}

fn zeta_k_fn<B: BitvectorBound>(
    left_min: UnsignedBitvector<B>,
    left_max: UnsignedBitvector<B>,
    right_min: UnsignedBitvector<B>,
    right_max: UnsignedBitvector<B>,
    k: u32,
    bound_func: fn(u32) -> u32,
    func: fn(ConcreteBitvector<RBound>, ConcreteBitvector<RBound>) -> ConcreteBitvector<RBound>,
) -> (ConcreteBitvector<RBound>, ConcreteBitvector<RBound>) {
    let bound = left_min.bound();
    let width = bound.width();
    let result_bound = RBound::new(bound_func(width));
    if width == 0 {
        let zero = ConcreteBitvector::new_zero(result_bound);
        return (zero.clone(), zero);
    }

    let mut lhs_min = left_min.cast_bitvector().uext(result_bound);
    let mut lhs_max = left_max.cast_bitvector().uext(result_bound);
    let mut rhs_min = right_min.cast_bitvector().uext(result_bound);
    let mut rhs_max = right_max.cast_bitvector().uext(result_bound);

    // set all bits above the interval [0, k] to zero
    let lo = k + 1;
    let hi = width - 1;
    if lo <= hi {
        lhs_min.set_bits(lo, hi, false);
        lhs_max.set_bits(lo, hi, false);
        rhs_min.set_bits(lo, hi, false);
        rhs_max.set_bits(lo, hi, false);
    }

    let k = ConcreteBitvector::from_u32(k, result_bound);

    // shift right, using the overflow as well
    let zeta_k_min = func(lhs_min, rhs_min).logic_shr(k.clone());
    let zeta_k_max = func(lhs_max, rhs_max).logic_shr(k);

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
