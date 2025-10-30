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

    fn udiv(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return zero as by SMT-LIB2
            return ConcreteBitvector::new(0, self.bound);
        }
        let result = dividend
            .checked_div(divisor)
            .expect("Unsigned division should only return none on zero divisor");

        Self::from_masked_u64(result, self.bound)
    }

    fn urem(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_u64();
        let divisor = rhs.to_u64();
        if divisor == 0 {
            // return zero as by SMT-LIB2
            return ConcreteBitvector::new(0, self.bound);
        }
        let result = dividend
            .checked_rem(divisor)
            .expect("Unsigned remainder should only return none on zero divisor");
        Self::from_masked_u64(result, self.bound)
    }

    fn sdiv(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return zero as by SMT-LIB2
            return ConcreteBitvector::new(0, self.bound);
        }

        // wrapping div as by SMT-LIB2
        let result = dividend.wrapping_div(divisor);
        Self::from_masked_u64(result as u64, self.bound)
    }

    fn srem(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);

        let dividend = self.to_i64();
        let divisor = rhs.to_i64();
        if divisor == 0 {
            // return zero as by SMT-LIB2
            return ConcreteBitvector::new(0, self.bound);
        }
        let signed_minus_one = self.bound.mask();
        let signed_minimum = self.bound.sign_bit_mask();
        if self.value == signed_minimum && rhs.value == signed_minus_one {
            // return zero as by SMT-LIB2
            return ConcreteBitvector::new(0, self.bound);
        }
        // wrapping rem as by SMT-LIB2
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
