use std::collections::BTreeMap;

use super::{super::LinearSlice, LinearPolynomial};
use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

impl LinearPolynomial {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let mut value = D::single_value(self.constant_term);
        let polynomial_bound = value.bound();

        for monomial in &self.linear_terms {
            let slice = monomial.slice;

            let mut formula_value = (fetch)(slice.formula_id);
            let bound = formula_value.bound();
            // slice
            // first, unsigned shift right to lsb if nonzero
            if slice.lsb != 0 {
                let lsb = ConcreteBitvector::new(slice.lsb.into(), bound);
                formula_value = formula_value.logic_shr(D::single_value(lsb));
            }

            let slice_bound = RBound::new(slice.width.get());

            // perform unsigned extension to slice width unless it is a no-op
            if formula_value.bound() != slice_bound {
                formula_value = formula_value.uext(slice_bound);
            }

            // unless the slice width is equal to polynomial width,
            // perform unsigned extension to it
            if slice_bound != polynomial_bound {
                formula_value = formula_value.uext(polynomial_bound);
            }

            // then, multiply by the coefficient
            let term_value = formula_value.mul(D::single_value(monomial.coefficient));
            value = value.add(term_value);
        }

        value
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.linear_terms
            .iter()
            .map(|monomial| monomial.slice.formula_id)
            .collect()
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        let remap = |slice: LinearSlice| {
            let Some(new_id) = old_to_new.get(&slice.formula_id) else {
                panic!(
                    "Used formula id {:?} should be remappable",
                    slice.formula_id
                );
            };
            LinearSlice {
                formula_id: *new_id,
                lsb: slice.lsb,
                width: slice.width,
            }
        };

        for monomial in &mut self.linear_terms {
            monomial.slice = remap(monomial.slice);
        }
    }
}
