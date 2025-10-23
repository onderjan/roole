use super::ThreeValued;
use crate::bitvector::concr::RUnsigned;

impl<T: RUnsigned> ThreeValued<T> {
    fn arith_neg(self, width: T::Width) -> Self {
        // arithmetic negation
        // since we use wrapping arithmetic, same as subtracting the value from 0

        let zero = Self::new(T::zero(width), width);
        Self::sub(zero, self, width)
    }
    pub fn add(self, rhs: Self, width: T::Width) -> Self {
        Self::minmax_compute(self, rhs, width, |lhs, rhs, k| {
            Self::addsub_zeta_k_fn(
                lhs.umin(width),
                lhs.umax(width),
                rhs.umin(width),
                rhs.umax(width),
                k,
                |lhs, rhs| lhs.add_shr(rhs, k, width),
            )
        })
    }
    pub fn sub(self, rhs: Self, width: T::Width) -> Self {
        Self::minmax_compute(self, rhs, width, |lhs, rhs, k| {
            // swap rhs min and max as it is applied in negative
            Self::addsub_zeta_k_fn(
                lhs.umin(width),
                lhs.umax(width),
                rhs.umax(width),
                rhs.umin(width),
                k,
                |lhs, rhs| lhs.sub_shr(rhs, k, width),
            )
        })
    }
    /*fn mul(self, rhs: Self, width: u32) -> Self {
        // use the minmax algorithm for now
        minmax_compute(self, rhs, |lhs, rhs, k| {
            // prepare a mask that selects interval [0, k]
            let mod_mask = util::compute_u64_mask(k + 1);

            // convert all to u128 so there is no overflow
            let left_min = (lhs.umin().to_u64() & mod_mask) as u128;
            let right_min = (rhs.umin().to_u64() & mod_mask) as u128;
            let left_max = (lhs.umax().to_u64() & mod_mask) as u128;
            let right_max = (rhs.umax().to_u64() & mod_mask) as u128;

            let zeta_k_min = ((left_min * right_min) >> k) as u64;
            let zeta_k_max = ((left_max * right_max) >> k) as u64;
            (zeta_k_min, zeta_k_max)
        })
    }*/

    fn minmax_compute(
        lhs: ThreeValued<T>,
        rhs: ThreeValued<T>,
        width: T::Width,
        zeta_k_fn: impl Fn(ThreeValued<T>, ThreeValued<T>, T::Index) -> (T, T),
    ) -> ThreeValued<T> {
        // from previous paper

        // start with no possibilites
        let mut zeros = T::zero(width);
        let mut ones = T::zero(width);

        // iterate over output bits
        for k in T::index_iter(width) {
            // compute h_k extremes
            let (zeta_k_min, zeta_k_max) = zeta_k_fn(lhs, rhs, k);

            let index_flag = T::index_flag(k);

            // see if minimum and maximum differs
            if zeta_k_min != zeta_k_max {
                // set result bit unknown
                zeros = zeros.bitor(index_flag, width);
                ones = ones.bitor(index_flag, width);
            } else {
                // set value of bit k, converted to ones-zeros encoding
                let k_is_one = zeta_k_min.bitand(index_flag, width) != T::zero(width);
                if k_is_one {
                    ones = ones.bitor(index_flag, width);
                } else {
                    zeros = zeros.bitor(index_flag, width);
                }
            }
            println!(
                "Computed k, min: {:?}, max: {:?}, zeros: {:?}, ones: {:?}",
                zeta_k_min, zeta_k_max, zeros, ones
            );
        }
        ThreeValued::from_zeros_ones(zeros, ones, width)
    }

    fn addsub_zeta_k_fn(
        left_min: T,
        left_max: T,
        right_min: T,
        right_max: T,
        k: T::Index,
        func: impl Fn(T, T) -> T,
    ) -> (T, T) {
        // prepare a mask that selects interval [0, k]
        let mod_mask = T::width_up_to(k);

        let left_min = left_min.limited(mod_mask);
        let left_max = left_max.limited(mod_mask);
        let right_min = right_min.limited(mod_mask);
        let right_max = right_max.limited(mod_mask);

        // shift right, using the overflow as well
        let zeta_k_min = func(left_min, right_min);
        let zeta_k_max = func(left_max, right_max);

        println!(
            "Left [{:?}, {:?}], right [{:?}, {:?}]",
            left_min, left_max, right_min, right_max
        );

        (zeta_k_min, zeta_k_max)
    }
}

/*fn shr_overflowing(overflowing_result: (u64, bool), k: u32) -> u64 {
    let mut result = overflowing_result.0 >> k;
    if overflowing_result.1 && k > 0 {
        let overflow_pos = u64::BITS - k;
        result |= 1u64 << overflow_pos;
    }
    result
}*/

/*fn convert_uarith<const W: u32>(min: u64, max: u64) -> ThreeValued<W> {
    // make highest different bit and all after it unknown
    let different = min ^ max;
    if different == 0 {
        // both are the same
        return ThreeValued::new(min);
    }

    let highest_different_bit_pos = different.ilog2();
    let unknown_mask = util::compute_u64_mask(highest_different_bit_pos + 1);
    ThreeValued::new_value_unknown(
        ConcreteBitvector::new(min),
        ConcreteBitvector::new(unknown_mask),
    )
}

fn compute_sdivrem<const W: u32>(
    dividend: ThreeValued<W>,
    divisor: ThreeValued<W>,
    op_fn: fn(SignedBitvector<W>, SignedBitvector<W>) -> SignedBitvector<W>,
) -> ThreeValued<W> {
    if W == 0 {
        // prevent problems
        return dividend;
    }

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
            SignedBitvector::new(1)
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
            SignedBitvector::new(0),
            SignedBitvector::new(0),
            op_fn,
        );
    }

    if divisor_min.to_i64() <= -1 && divisor_max.to_i64() >= -1 {
        // -1 divisor, causes overflow when the dividend is the most negative value, handle separately
        // handle separately

        let minus_one = ConcreteBitvector::bit_mask().cast_signed();

        let mut dividend_min = dividend.smin();
        let dividend_max = dividend.smax();

        if dividend_min == ConcreteBitvector::sign_bit_mask().cast_signed() {
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
                dividend_min = dividend_min + SignedBitvector::new(1);
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
            -SignedBitvector::new(2)
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

    ThreeValued::from_zeros_ones(ConcreteBitvector::new(zeros), ConcreteBitvector::new(ones))
}

fn apply_signed_op<const W: u32>(
    zeros: &mut u64,
    ones: &mut u64,
    a_min: SignedBitvector<W>,
    a_max: SignedBitvector<W>,
    b_min: SignedBitvector<W>,
    b_max: SignedBitvector<W>,
    op_fn: fn(SignedBitvector<W>, SignedBitvector<W>) -> SignedBitvector<W>,
) {
    // apply all configurations
    // cast to unsigned u64 afterwards
    let x = op_fn(a_min, b_min).as_bitvector().cast_unsigned().to_u64();
    let y = op_fn(a_min, b_max).as_bitvector().cast_unsigned().to_u64();
    let z = op_fn(a_max, b_min).as_bitvector().cast_unsigned().to_u64();
    let w = op_fn(a_max, b_max).as_bitvector().cast_unsigned().to_u64();

    // find the highest different bit
    let found_zeros = (!x | !y | !z | !w) & util::compute_u64_mask(W);
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
    let unknown_mask = util::compute_u64_mask(highest_different_bit_pos + 1);

    *zeros |= unknown_mask;
    *ones |= unknown_mask;
}
*/
