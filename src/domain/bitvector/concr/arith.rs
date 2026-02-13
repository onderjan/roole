use crate::domain::{bitvector::BitvectorBound, traits::forward::HwArith};

use super::ConcreteBitvector;

impl<B: BitvectorBound> HwArith for ConcreteBitvector<B> {
    fn arith_neg(self) -> Self {
        let result = self.value.wrapping_neg();
        Self::from_masked_u64(result, self.bound)
    }

    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value.wrapping_add(rhs.value);
        Self::from_masked_u64(result, self.bound)
    }

    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value.wrapping_sub(rhs.value);
        Self::from_masked_u64(result, self.bound)
    }

    fn mul(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let result = self.value.wrapping_mul(rhs.value);
        Self::from_masked_u64(result, self.bound)
    }

    fn udiv_wrapping_or_full(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        if rhs.is_zero() {
            // return full bitvector
            return ConcreteBitvector::new_umax(self.bound);
        }

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        let result = dividend.wrapping_div(divisor);

        Self::from_masked_u64(result, self.bound)
    }

    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        if rhs.is_zero() {
            // return dividend
            return self;
        }

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        let result = dividend.wrapping_rem(divisor);
        Self::from_masked_u64(result, self.bound)
    }

    fn sdiv_wrapping_or_full(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        if rhs.is_zero() {
            // return full bitvector
            return ConcreteBitvector::new_umax(self.bound);
        }

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();

        let result = dividend.wrapping_div(divisor);
        Self::from_masked_u64(result as u64, self.bound)
    }

    fn srem_wrapping_or_dividend(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        if rhs.is_zero() {
            // return dividend
            return self;
        }

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();

        let result = dividend.wrapping_rem(divisor);
        Self::from_masked_u64(result as u64, self.bound)
    }
}

/*impl<B: BitvectorBound> ConcreteBitvector<B> {
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
}*/
