use std::collections::BTreeMap;
use std::fmt::Debug;

use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::formula::FormulaId,
};

mod ops;

/// A linear combination of bitvectors and a constant.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    pub constant: ConcreteBitvector<RBound>,
    pub monomials: BTreeMap<FormulaId, ConcreteBitvector<RBound>>,
}

impl LinearCombination {
    pub fn from_constant(constant: ConcreteBitvector<RBound>) -> Self {
        Self {
            constant,
            monomials: BTreeMap::new(),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.monomials.keys().copied().collect()
    }

    pub fn bound(&self) -> RBound {
        self.constant.bound()
    }

    pub(super) fn normalize(&mut self) {
        // eliminate zero coefficients
        self.monomials.retain(|_, coeff| !coeff.is_zero());
    }

    pub fn remap(&mut self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) {
        let remap = |formula_id| {
            let Some(new_id) = old_to_new.get_by_left(&formula_id) else {
                panic!("Used formula id {:?} should be remappable", formula_id);
            };
            *new_id
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
}

impl Debug for LinearCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;

        write!(f, "(")?;

        // write the linear combinations of formulas with coefficients
        for (formula_id, coefficient) in &self.monomials {
            if is_first {
                is_first = false;
            } else {
                write!(f, " + ")?;
            }

            let one = ConcreteBitvector::<RBound>::one(coefficient.bound());

            if coefficient != &one {
                write!(f, "{}*", coefficient)?;
            }

            write!(f, "{:?}", formula_id)?;
        }

        if is_first {
            write!(f, "{}", self.constant)?;
        } else if self.constant.is_nonzero() {
            write!(f, " + {}", self.constant)?;
        }
        write!(f, ") mod {}", 1u128 << self.constant.bound().width())
    }
}
