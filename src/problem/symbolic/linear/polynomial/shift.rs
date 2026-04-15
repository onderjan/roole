use std::num::NonZero;

use itertools::Itertools;

use super::LinearPolynomial;
use crate::domain::{bitvector::concr::ConcreteBitvector, traits::forward::HwShift};

impl LinearPolynomial {
    pub fn logic_shl(mut self, amount: Self) -> Result<Self, ()> {
        let bound = self.bound();
        assert_eq!(self.bound(), amount.bound());

        let Some(amount) = amount.constant_value() else {
            return Err(());
        };

        // TODO: consider whether to mask amounts or not, this masks them

        // logical shift left with a constant value is just scaling
        // by the given power of 2

        let multiplier = if let Some(amount) = amount.try_to_u32() {
            ConcreteBitvector::single_bit(amount, bound)
        } else {
            // the shift will inevitably shift everything out, this is the same as scaling by zero
            ConcreteBitvector::new_zero(bound)
        };

        self.scale(multiplier);

        Ok(self)
    }

    pub fn logic_shr(mut self, amount: Self) -> Result<Self, ()> {
        let bound = self.bound();
        assert_eq!(self.bound(), amount.bound());

        let Some(amount) = amount.constant_value() else {
            return Err(());
        };

        if self.might_overflow() {
            return Err(());
        }

        // amount is constant and the polynomial cannot overflow
        if self.linear_terms.is_empty() {
            // we can simply shift the constant right by the amount
            self.constant_term = self.constant_term.logic_shr(amount);
            return Ok(self);
        }

        let Ok(mut monomial) = self.linear_terms.into_iter().exactly_one() else {
            return Err(());
        };

        // TODO: handle other coefficients
        if !monomial.coefficient.is_one() {
            return Err(());
        }

        let Some(amount) = amount.try_to_u32() else {
            // the shift amount is greater than maximum representable width
            // this will clearly make the polynomial empty
            return Ok(Self::empty(bound));
        };

        // our polynomial only contains the slice

        // TODO: consider whether to mask amounts or not, this does not mask them

        if amount < monomial.slice.width.get() {
            // we will drop the lowest bits by increasing lsb
            // the width must decrease correspondingly
            monomial.slice.lsb += amount;
            monomial.slice.width = NonZero::new(monomial.slice.width.get() - amount)
                .expect("Slice width should be nonzero after logical shift right");

            Ok(LinearPolynomial::from_monomial(monomial))
        } else {
            // all bits will be dropped
            Ok(Self::empty(bound))
        }
    }

    pub fn arith_shr(self, _amount: Self) -> Result<Self, ()> {
        // TODO: arithmetic shift right
        Err(())
    }
}
