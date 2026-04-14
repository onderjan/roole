use crate::domain::{bitvector::BitvectorBound, traits::forward::HwArith};

use super::ConcreteBitvector;

impl<B: BitvectorBound> HwArith for ConcreteBitvector<B> {
    fn arith_neg(self) -> Self {
        Self::from_masked(-self.value, self.bound)
    }

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked(self.value + rhs.value, self.bound)
    }

    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked(self.value - rhs.value, self.bound)
    }

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        Self::from_masked(self.value * rhs.value, self.bound)
    }

    fn udiv_wrapping_or_all_ones(self, rhs: Self) -> Self {
        todo!("Udiv");
        /*let bound = self.bound;
        assert_eq!(bound, rhs.bound);

        if rhs.is_zero() {
            // return full bitvector
            return ConcreteBitvector::new_all_ones(self.bound);
        }

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        let result = dividend.wrapping_div(divisor);

        Self::from_masked_u64(result, bound)*/
    }

    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self {
        todo!("Urem");
        /*let bound = self.bound;
        assert_eq!(bound, rhs.bound);

        if rhs.is_zero() {
            // return dividend
            return self;
        }

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        let result = dividend.wrapping_rem(divisor);
        Self::from_masked_u64(result, bound)*/
    }

    fn sdiv_wrapping_by_quadrants(self, rhs: Self) -> Self {
        todo!("Sdiv");
        /*
        let bound = self.bound;
        assert_eq!(bound, rhs.bound);

        if rhs.is_zero() {
            // the value to return depends on the sign of dividend
            return if self.is_sign_bit_set() {
                // return one
                ConcreteBitvector::new_one(bound)
            } else {
                // return all-ones
                ConcreteBitvector::new_all_ones(bound)
            };
        }

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();

        let result = dividend.wrapping_div(divisor);
        Self::from_masked_u64(result as u64, bound)
        */
    }

    fn srem_wrapping_by_quadrants(self, rhs: Self) -> Self {
        todo!("Srem");
        /*
        let bound = self.bound;
        assert_eq!(bound, rhs.bound);

        if rhs.is_zero() {
            // return dividend
            return self;
        }

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();

        let result = dividend.wrapping_rem(divisor);
        Self::from_masked_u64(result as u64, bound)
        */
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
