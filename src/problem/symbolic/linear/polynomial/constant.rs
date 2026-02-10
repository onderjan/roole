use itertools::Itertools;

use super::{LinearMonomial, LinearPolynomial};
use crate::domain::{
    bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
    traits::forward::HwArith,
};

impl LinearPolynomial {
    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        if self.linear_terms.is_empty() {
            Some(self.constant_term)
        } else {
            None
        }
    }

    pub fn constant_value_with_assumption(
        &self,
        assumption: &LinearPolynomial,
    ) -> Option<ConcreteBitvector<RBound>> {
        if let Some(constant_value) = self.constant_value() {
            return Some(constant_value);
        }

        if assumption.bound().width() != 1 {
            return None;
        }

        let Ok((assumption_slice, assumption_factor)) =
            assumption.linear_terms.iter().exactly_one()
        else {
            return None;
        };

        if assumption_slice.width.get() != 1 || !assumption_factor.is_one() {
            return None;
        }

        let Ok((our_slice, our_factor)) = self.linear_terms.iter().exactly_one() else {
            return None;
        };

        if our_slice != assumption_slice {
            return None;
        }

        let mut result = self.constant_term;

        let slice_holds = assumption.constant_term.is_zero();

        if slice_holds {
            result = result.add(*our_factor);
        }

        Some(result)
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
