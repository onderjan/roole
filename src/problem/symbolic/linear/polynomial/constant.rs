use itertools::Itertools;

use super::{LinearMonomial, LinearPolynomial};
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::{BExt, HwArith, HwShift},
    },
    problem::symbolic::linear::slice::LinearSlice,
};

impl LinearPolynomial {
    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        if self.linear_terms.is_empty() {
            Some(self.constant_term)
        } else {
            None
        }
    }

    pub fn constant_value_assuming(
        &self,
        assumption: &LinearPolynomial,
    ) -> Option<ConcreteBitvector<RBound>> {
        if assumption.bound().width() != 1 {
            return None;
        }

        let Ok((assumption_slice, assumption_factor)) =
            assumption.linear_terms.iter().exactly_one()
        else {
            return None;
        };

        if !assumption_factor.is_one() {
            return None;
        }

        let value = assumption.constant_term.arith_neg();

        let mut polynomial = self.clone();
        polynomial.assume(*assumption_slice, value);
        polynomial.constant_value()
    }

    pub fn assume(&mut self, assumed_slice: LinearSlice, assumed_value: ConcreteBitvector<RBound>) {
        let bound = self.bound();
        let mut remove_slices = Vec::new();

        for (slice, factor) in self.linear_terms.iter() {
            if slice.formula_id != assumed_slice.formula_id {
                continue;
            }

            if !assumed_slice.contains(slice) {
                continue;
            }

            let mut slice_value = assumed_value;

            if slice.lsb > assumed_slice.lsb {
                // unsigned-shift assumed value right to drop bits below slice lsb
                let amount = slice.lsb - assumed_slice.lsb;
                let amount = ConcreteBitvector::new(amount.into(), slice_value.bound());
                slice_value = slice_value.logic_shr(amount);
            }

            let slice_width = slice.width.get();

            if slice_value.bound().width() != slice_width {
                // unsigned-extend to slice width
                slice_value = slice_value.uext(RBound::new(slice_width));
            }

            if slice_value.bound() != bound {
                // unsigned-extend to our width
                slice_value = slice_value.uext(bound);
            }

            self.constant_term = self.constant_term.add(slice_value.mul(*factor));
            remove_slices.push(*slice);
        }

        for remove_slice in remove_slices {
            self.linear_terms.remove(&remove_slice);
        }
    }

    pub fn monomial_and_constant_value(
        &self,
    ) -> Option<(Option<LinearMonomial>, ConcreteBitvector<RBound>)> {
        if self.linear_terms.is_empty() {
            return Some((None, self.constant_term));
        }
        let Ok((slice, coefficient)) = self.linear_terms.iter().exactly_one() else {
            return None;
        };

        Some((
            Some(LinearMonomial::new(*coefficient, *slice)),
            self.constant_term,
        ))
    }
}
