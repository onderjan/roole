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

        let Ok(assumption_monomial) = assumption.linear_terms.iter().exactly_one() else {
            return None;
        };

        if !assumption_monomial.coefficient.is_one() {
            return None;
        }

        let assumption_slice = assumption_monomial.slice;

        let value = assumption.constant_term.arith_neg();

        let mut polynomial = self.clone();
        polynomial.assume(assumption_slice, value);
        polynomial.constant_value()
    }

    pub fn assume(&mut self, assumed_slice: LinearSlice, assumed_value: ConcreteBitvector<RBound>) {
        let bound = self.bound();

        // for each linear term, either convert it to a constant or retain it
        self.linear_terms.retain(|monomial| {
            let slice = monomial.slice;

            if slice.formula_id != assumed_slice.formula_id {
                // retain
                return true;
            }

            if !assumed_slice.contains(&slice) {
                // retain
                return true;
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

            // convert to constant term
            self.constant_term = self
                .constant_term
                .add(slice_value.mul(monomial.coefficient));
            false
        });
    }

    pub fn monomial_and_constant_value(
        &self,
    ) -> Option<(Option<LinearMonomial>, ConcreteBitvector<RBound>)> {
        if self.linear_terms.is_empty() {
            return Some((None, self.constant_term));
        }
        let Ok(monomial) = self.linear_terms.iter().exactly_one() else {
            return None;
        };

        Some((Some(monomial.clone()), self.constant_term))
    }
}
