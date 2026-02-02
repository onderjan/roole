use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

mod ops;

/// A linear combination of bitvectors and a constant.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    constant: ConcreteBitvector<RBound>,
    monomials: BTreeMap<FormulaId, ConcreteBitvector<RBound>>,
}

impl LinearCombination {
    pub fn new(
        constant: ConcreteBitvector<RBound>,
        monomials: BTreeMap<FormulaId, ConcreteBitvector<RBound>>,
    ) -> Self {
        let mut result = Self {
            constant,
            monomials,
        };
        result.normalize();
        result
    }

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

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let mut value = D::single_value(self.constant);
        for (formula_id, coefficient) in &self.monomials {
            let formula_value = (fetch)(*formula_id);
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
        let remap = |formula_id| {
            let Some(new_id) = old_to_new.get(&formula_id) else {
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

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        if self.monomials.is_empty() {
            Some(self.constant)
        } else {
            None
        }
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        domain::bitvector::{RBound, concr::ConcreteBitvector},
        problem::formula::{FormulaId, VariableId},
    };

    use super::*;

    #[test]
    fn test_addsub() {
        let bound = RBound::new(32);
        let a = LinearCombination {
            constant: ConcreteBitvector::new(38, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(12, bound),
            )]),
        };
        let b = LinearCombination {
            constant: ConcreteBitvector::new(17, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(7, bound),
            )]),
        };
        let add_result = LinearCombination {
            constant: ConcreteBitvector::new(55, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(19, bound),
            )]),
        };
        let sub_result = LinearCombination {
            constant: ConcreteBitvector::new(21, bound),
            monomials: BTreeMap::from_iter([(
                FormulaId::Variable(VariableId(0)),
                ConcreteBitvector::new(5, bound),
            )]),
        };
        assert_eq!(a.clone().add(b.clone()), add_result);
        assert_eq!(a.sub(b), sub_result);
    }
}
