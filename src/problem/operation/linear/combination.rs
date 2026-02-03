use std::collections::BTreeMap;
use std::fmt::Debug;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::problem::operation::linear::slice::LinearSlice;
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

mod arith;
mod ext;
mod shift;

/// A linear combination of bitvectors and a constant.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    constant: ConcreteBitvector<RBound>,
    monomials: BTreeMap<LinearSlice, ConcreteBitvector<RBound>>,
}

impl LinearCombination {
    pub fn new(
        constant: ConcreteBitvector<RBound>,
        monomials: BTreeMap<LinearSlice, ConcreteBitvector<RBound>>,
    ) -> Self {
        let mut result = Self {
            constant,
            monomials,
        };
        result.normalize();
        result
    }

    pub fn empty(bound: RBound) -> Self {
        Self {
            constant: ConcreteBitvector::zero(bound),
            monomials: BTreeMap::new(),
        }
    }

    pub fn from_constant(constant: ConcreteBitvector<RBound>) -> Self {
        Self {
            constant,
            monomials: BTreeMap::new(),
        }
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        let mut monomials = BTreeMap::new();

        if let Some(slice) = LinearSlice::from_bounded(formula_id, bound) {
            monomials.insert(slice, ConcreteBitvector::one(bound));
        }

        LinearCombination::new(ConcreteBitvector::zero(bound), monomials)
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.monomials
            .keys()
            .map(|slice| slice.formula_id)
            .collect()
    }

    pub fn bound(&self) -> RBound {
        self.constant.bound()
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let mut value = D::single_value(self.constant);
        let combination_bound = value.bound();
        let combination_width = combination_bound.width();

        for (slice, coefficient) in &self.monomials {
            let mut formula_value = (fetch)(slice.formula_id);
            let bound = formula_value.bound();
            // slice
            // first, unsigned shift right to lsb if nonzero
            if slice.lsb != 0 {
                let lsb = ConcreteBitvector::new(slice.lsb.into(), bound);
                formula_value = formula_value.logic_shr(D::single_value(lsb));
            }

            // unless slice lsb is equal to zero and slice bound width is equal to width,
            // perform unsigned extension
            if slice.lsb != 0 || slice.width.get() != combination_width {
                formula_value = formula_value.uext(combination_bound);
            }

            // then, multiply by the coefficient
            let term_value = formula_value.mul(D::single_value(*coefficient));
            value = value.add(term_value);
        }

        value
    }

    pub(super) fn normalize(&mut self) {
        // eliminate zero coefficients
        self.monomials.retain(|_, coeff| !coeff.is_zero());
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

        let mut old_monomials = BTreeMap::new();
        std::mem::swap(&mut self.monomials, &mut old_monomials);

        for (formula_id, coefficient) in old_monomials {
            self.monomials.insert(remap(formula_id), coefficient);
        }
    }

    pub fn scale(&mut self, scaler: ConcreteBitvector<RBound>) {
        let bound = self.bound();
        assert_eq!(bound, scaler.bound());

        self.constant = self.constant.mul(scaler);

        for coefficient in self.monomials.values_mut() {
            *coefficient = coefficient.mul(scaler);
        }
    }

    pub fn single_bit(constant: bool) -> LinearCombination {
        let bound = RBound::single_bit_bound();
        let constant = if constant {
            ConcreteBitvector::one(bound)
        } else {
            ConcreteBitvector::zero(bound)
        };

        LinearCombination::from_constant(constant)
    }

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        if self.monomials.is_empty() {
            Some(self.constant)
        } else {
            None
        }
    }

    pub fn might_overflow(&self) -> bool {
        if self.monomials.is_empty() {
            // only constant, definitely cannot overflow
            return false;
        }

        // TODO: determine if the combination might overflow more finely

        if self.constant.is_zero()
            && let Ok((slice, factor)) = self.monomials.iter().exactly_one()
        {
            let emplaced_slice_width = slice.lsb + slice.width.get();
            let slice_fits = emplaced_slice_width <= self.bound().width();
            if factor.is_one() && slice_fits {
                // only one monomial that fits, definitely cannot overflow
                return false;
            }
        }
        // if we are unsure, return true
        true
    }
}

impl Debug for LinearCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;

        write!(f, "(")?;

        // write the linear combinations of formulas with coefficients
        for (slice, coefficient) in &self.monomials {
            if is_first {
                is_first = false;
            } else {
                write!(f, " + ")?;
            }

            let one = ConcreteBitvector::<RBound>::one(coefficient.bound());

            if coefficient != &one {
                write!(f, "{}*", coefficient)?;
            }

            write!(f, "{:?}", slice)?;
        }

        if is_first {
            write!(f, "{}", self.constant)?;
        } else if self.constant.is_nonzero() {
            write!(f, " + {}", self.constant)?;
        }
        write!(f, ") mod {}", 1u128 << self.constant.bound().width())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, num::NonZero};

    use crate::{
        domain::bitvector::{RBound, concr::ConcreteBitvector},
        problem::formula::{FormulaId, VariableId},
    };

    use super::*;

    #[test]
    fn test_addsub() {
        let bound = RBound::new(32);
        let slice = LinearSlice {
            formula_id: FormulaId::Variable(VariableId(0)),
            lsb: 0,
            width: NonZero::new(32).unwrap(),
        };
        let a = LinearCombination {
            constant: ConcreteBitvector::new(38, bound),
            monomials: BTreeMap::from_iter([(slice, ConcreteBitvector::new(12, bound))]),
        };
        let b = LinearCombination {
            constant: ConcreteBitvector::new(17, bound),
            monomials: BTreeMap::from_iter([(slice, ConcreteBitvector::new(7, bound))]),
        };
        let add_result = LinearCombination {
            constant: ConcreteBitvector::new(55, bound),
            monomials: BTreeMap::from_iter([(slice, ConcreteBitvector::new(19, bound))]),
        };
        let sub_result = LinearCombination {
            constant: ConcreteBitvector::new(21, bound),
            monomials: BTreeMap::from_iter([(slice, ConcreteBitvector::new(5, bound))]),
        };
        assert_eq!(a.clone().add(b.clone()), add_result);
        assert_eq!(a.sub(b), sub_result);
    }
}
