use crate::domain::{bitvector::BitvectorBound, traits::forward::HwArith};

use super::ConcreteBitvector;

impl<B: BitvectorBound> HwArith for ConcreteBitvector<B> {
    fn arith_neg(self) -> Self {
        let value = self
            .value
            .uni_upwards(|a, borrow| 0u64.borrowing_sub(a, borrow), false);
        Self::from_masked(value, self.bound)
    }

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let value = self
            .value
            .bi_upwards(rhs.value, |a, b, carry| a.carrying_add(b, carry), false);
        Self::from_masked(value, self.bound)
    }

    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let value =
            self.value
                .bi_upwards(rhs.value, |a, b, borrow| a.borrowing_sub(b, borrow), false);
        Self::from_masked(value, self.bound)
    }

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked(self.value.mul(rhs.value), self.bound)
    }

    fn udiv_wrapping_or_all_ones(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let value = self.value.udiv_wrapping_or_all_ones(rhs.value, self.bound);
        Self::from_masked(value, self.bound)
    }

    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self {
        let value = self.value.urem_wrapping_or_dividend(rhs.value, self.bound);
        Self::from_masked(value, self.bound)
    }

    fn sdiv_wrapping_by_quadrants(mut self, mut rhs: Self) -> Self {
        let negative_lhs = self.is_sign_bit_set();
        let negative_rhs = rhs.is_sign_bit_set();

        if negative_lhs {
            self = self.arith_neg();
        }
        if negative_rhs {
            rhs = rhs.arith_neg();
        }

        let mut result = self.udiv_wrapping_or_all_ones(rhs);

        // negate the result exactly if just one is negative
        if negative_lhs ^ negative_rhs {
            result = result.arith_neg();
        }

        result
    }

    fn srem_wrapping_by_quadrants(mut self, mut rhs: Self) -> Self {
        let negative_lhs = self.is_sign_bit_set();
        let negative_rhs = rhs.is_sign_bit_set();

        if negative_lhs {
            self = self.arith_neg();
        }
        if negative_rhs {
            rhs = rhs.arith_neg();
        }

        let mut result = self.urem_wrapping_or_dividend(rhs);

        // negate the result exactly if the dividend is negative
        if negative_lhs {
            result = result.arith_neg();
        }

        result
    }

    fn smod_wrapping_by_quadrants(self, rhs: Self) -> Self {
        // implemented according to https://smt-lib.org/logics-all.shtml#QF_BV
        let negative_lhs = self.is_sign_bit_set();
        let negative_rhs = rhs.is_sign_bit_set();

        let abs_lhs = if !negative_lhs {
            self
        } else {
            self.arith_neg()
        };

        let abs_rhs = if !negative_rhs {
            rhs.clone()
        } else {
            rhs.clone().arith_neg()
        };

        let u = abs_lhs.urem_wrapping_or_dividend(abs_rhs);
        if u.is_zero() {
            return u;
        }
        match (negative_lhs, negative_rhs) {
            (false, false) => u,
            (true, false) => u.arith_neg().add(rhs),
            (false, true) => u.add(rhs),
            (true, true) => u.arith_neg(),
        }
    }
}

fn udiv_wrapping_or_all_ones_u64(a: u64, b: u64) -> u64 {
    if b != 0 {
        a.wrapping_div(b)
    } else {
        // return all ones
        u64::MAX
    }
}

fn urem_wrapping_or_dividend_u64(a: u64, b: u64) -> u64 {
    if b != 0 {
        a.wrapping_rem(b)
    } else {
        // return dividend
        a
    }
}

/*
impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub(crate) fn checked_add(self, rhs: Self) -> Option<Self> {
        assert_eq!(self.bound, rhs.bound);

        let result = self.value.checked_add(rhs.value)?;
        if result & !self.bound.mask() != 0 {
            return None;
        }
        Some(Self::new(result, self.bound))
    }

    pub(crate) fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.value.checked_mul(rhs.value)?;
        if result & !self.bound.mask() != 0 {
            return None;
        }
        Some(Self::new(result, self.bound))
    }
}
*/
