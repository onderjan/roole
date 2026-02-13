use itertools::Itertools;

use super::{
    super::{LinearMonomial, LinearSlice},
    LinearPolynomial,
};
use crate::domain::{
    bitvector::concr::ConcreteBitvector,
    traits::forward::{Bitwise, HwArith, HwShift},
};

impl LinearPolynomial {
    pub fn bit_not(self) -> Self {
        let mut result = self.arith_neg();
        result.constant_term = result
            .constant_term
            .sub(ConcreteBitvector::new_one(result.bound()));
        result.into_normal_form()
    }

    pub fn bitwise_combine(
        self,
        rhs: LinearPolynomial,
        conjunction: bool,
    ) -> Result<LinearPolynomial, ()> {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (constant_operand, other_operand) = match (self.constant_value(), rhs.constant_value())
        {
            (None, None) => return Err(()),
            (None, Some(constant)) => (constant, self),
            (Some(constant), None) => (constant, rhs),
            (Some(lhs), Some(rhs)) => {
                let result = if conjunction {
                    lhs.bit_and(rhs)
                } else {
                    lhs.bit_or(rhs)
                };
                return Ok(LinearPolynomial::from_concrete(result));
            }
        };

        // we have a constant value to be bitwise-combined with a polynomial

        if !other_operand.constant_term.is_zero() {
            return Err(());
        }

        let Ok(monomial) = other_operand.linear_terms.into_iter().exactly_one() else {
            return Err(());
        };

        let coefficient = monomial.coefficient;
        if !coefficient.to_u64().is_power_of_two() {
            return Err(());
        }

        let coefficient_log2 = coefficient.to_u64().ilog2();
        let coefficient_log2 = ConcreteBitvector::new(coefficient_log2.into(), bound);

        // now, we have a constant value to be bitwise-combined with a bit-shifted slice
        // get the slice output mask
        let slice_output_mask = monomial.slice.output_mask(bound);

        // bit-shift left by the coefficient logarithm to get the monomial mask
        let monomial_mask = slice_output_mask.logic_shl(coefficient_log2);

        let (new_monomial_mask, new_constant) = if conjunction {
            // bitwise AND, retain the monomial mask only where the constant operand had ones
            // the new constant is zero, as was previously
            let new_monomial_mask = monomial_mask.bit_and(constant_operand);
            let new_constant = ConcreteBitvector::new_zero(bound);
            (new_monomial_mask, new_constant)
        } else {
            // bitwise OR, retain the monomial mask only where the constant operand had zeroes
            // the new constant is exactly the constant operand
            let new_monomial_mask = monomial_mask.bit_and(constant_operand.bit_not());
            (new_monomial_mask, constant_operand)
        };

        // unsigned-bit-shift right by the coefficient logarithm to get the new slice output mask
        let mut new_slice_output_mask = new_monomial_mask.logic_shr(coefficient_log2);

        // the new slice mask can have holes in it, leading to multiple slices
        // or even be zero, leading to no slices
        // start with the constant polynomial and add slices as long as we can extract them

        let mut new_polynomial = LinearPolynomial::from_concrete(new_constant);

        let one = ConcreteBitvector::new_one(bound);

        while new_slice_output_mask.is_nonzero() {
            // turn off the rightmost contiguous string of 1-bits
            // from Hacker's Delight Chapter 2
            let with_slice_turned_off = new_slice_output_mask
                .bit_or(new_slice_output_mask.sub(one))
                .add(one)
                .bit_and(new_slice_output_mask);

            let turned_off_slice = new_slice_output_mask.sub(with_slice_turned_off);

            // construct the monomial from the turned-off slice
            let turned_off_slice =
                LinearSlice::from_mask(monomial.slice.formula_id, turned_off_slice);

            let turned_off_lsb = ConcreteBitvector::new(turned_off_slice.lsb.into(), bound);

            // we must compensate possibly non-zero lsb of the turned-off slice
            // by shifting the coefficient by it
            let turned_off_coefficient = coefficient.logic_shl(turned_off_lsb);

            let turned_off_polynomial = LinearPolynomial::from_monomial(LinearMonomial::new(
                turned_off_coefficient,
                turned_off_slice,
            ));

            new_polynomial = new_polynomial.add(turned_off_polynomial);

            new_slice_output_mask = with_slice_turned_off;
        }

        // we are done
        Ok(new_polynomial)
    }
}
