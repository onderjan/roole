use std::num::NonZero;

use itertools::Itertools;

use crate::{
    domain::{bitvector::concr::ConcreteBitvector, traits::forward::Bitwise},
    problem::operation::{
        LinearPolynomial,
        linear::{monomial::LinearMonomial, slice::LinearSlice},
    },
};

impl LinearPolynomial {
    pub fn bitwise_combine(
        self,
        rhs: LinearPolynomial,
        conjunction: bool,
    ) -> Result<LinearPolynomial, ()> {
        let bound = self.bound();

        let (constant, other) = match (self.constant_value(), rhs.constant_value()) {
            (None, None) => return Err(()),
            (None, Some(constant)) => (constant, self),
            (Some(constant), None) => (constant, rhs),
            (Some(lhs), Some(rhs)) => {
                let result = if conjunction {
                    lhs.bit_and(rhs)
                } else {
                    lhs.bit_or(rhs)
                };
                return Ok(LinearPolynomial::from_constant(result));
            }
        };

        // TODO: this is just written offhand and maybe wrong in some cases

        if !other.constant_term.is_zero() {
            return Err(());
        }

        let Ok((slice, coefficient)) = other.linear_terms.into_iter().exactly_one() else {
            return Err(());
        };

        let coefficient = coefficient.to_u64();
        if !coefficient.is_power_of_two() {
            return Err(());
        }

        // constant and a slice with power-of-two coefficient
        // TODO: make this more powerful

        let constant = constant.to_u64();

        let slice_mask = match 1u64.checked_shl(slice.width.get() + 1) {
            Some(m) => m - 1,
            None => u64::MAX,
        };

        let placed_mask = slice_mask * coefficient;

        let new_placed_mask = if conjunction {
            placed_mask & constant
        } else {
            placed_mask & !constant
        };

        let new_constant = if conjunction {
            !placed_mask & constant
        } else {
            constant
        };

        let new_slice_mask = new_placed_mask / coefficient;

        let Some(added_lsb) = new_slice_mask.checked_ilog2() else {
            // just the constant
            return Ok(LinearPolynomial::from_constant(ConcreteBitvector::new(
                new_constant,
                bound,
            )));
        };

        let down_mask = new_slice_mask >> added_lsb;

        if !(down_mask + 1).is_power_of_two() {
            // not a single slice
            return Err(());
        }

        let new_slice_width = NonZero::new(down_mask.count_ones()).unwrap();

        let new_slice_lsb = slice.lsb + added_lsb;
        assert!(new_slice_lsb + new_slice_width.get() <= slice.lsb + slice.width.get());

        let new_coefficient = coefficient << added_lsb;

        let new_slice = LinearSlice {
            formula_id: slice.formula_id,
            lsb: new_slice_lsb,
            width: new_slice_width,
        };

        let new_constant = ConcreteBitvector::new(new_constant, bound);
        let new_coefficient = ConcreteBitvector::new(new_coefficient, bound);

        let new_monomial = LinearMonomial::new(new_coefficient, new_slice);
        let result = LinearPolynomial::from_monomial_and_constant(new_monomial, new_constant);

        Ok(result)
    }
}
