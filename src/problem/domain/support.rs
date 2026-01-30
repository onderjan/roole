use std::{collections::BTreeMap, fmt::Debug};

use bimap::BiBTreeMap;

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::{Join, forward::HwArith},
    },
    problem::{
        domain::{LinearBitvector, LinearCombination, LinearRelation, LinearSystem},
        formula::FormulaId,
    },
};

impl LinearBitvector {
    pub fn for_formula_id(formula_id: FormulaId, bound: RBound) -> Self {
        let constant = ConcreteBitvector::zero(bound);
        let monomials = BTreeMap::from_iter([(formula_id, ConcreteBitvector::one(bound))]);

        LinearBitvector::Combination(LinearCombination {
            constant,
            monomials,
        })
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self {
            LinearBitvector::Top(_) => vec![],
            LinearBitvector::Combination(combination) => combination.used_ids(),
            LinearBitvector::System(system) => system.used_ids(),
        }
    }
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

    pub fn remap(self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) -> Self {
        let remap = |formula_id| {
            let Some(new_id) = old_to_new.get_by_left(&formula_id) else {
                panic!("Used formula id {:?} should be remappable", formula_id);
            };
            *new_id
        };

        LinearCombination {
            constant: self.constant,
            monomials: BTreeMap::from_iter(
                self.monomials
                    .iter()
                    .map(|(formula_id, coeff)| (remap(*formula_id), *coeff)),
            ),
        }
    }

    pub fn apply_fixed_mult(&mut self, fixed: ConcreteBitvector<RBound>) {
        let bound = self.bound();
        assert_eq!(bound, fixed.bound());

        self.constant = self.constant.mul(fixed);

        for coefficient in self.monomials.values_mut() {
            *coefficient = coefficient.mul(fixed);
        }
    }

    pub fn single_bit(is_one: bool) -> LinearCombination {
        let bound = RBound::single_bit_bound();
        let constant = if is_one {
            ConcreteBitvector::one(bound)
        } else {
            ConcreteBitvector::zero(bound)
        };

        LinearCombination::from_constant(constant)
    }
}

impl LinearSystem {
    pub fn normalize(&mut self) {
        eprintln!("Normalizing system: {:?}", self);

        // TODO: normalize with slack
        /*for relation in self.relations.iter_mut() {
            if let Some((first_formula_id, first_coeff)) =
                relation.combination.coefficients.first_key_value()
            {
                eprintln!("First: {}*{:?}", first_coeff, first_formula_id);
                if let Some(inverse_coeff) = first_coeff.modular_inverse() {
                    // multiply by the inverse coefficient
                    // the right side is zero, no need to multiply it
                    relation.combination.apply_fixed_mult(inverse_coeff);
                } else {
                    // TODO: do something without modular inverse
                }
            } else {
                // TODO: turn system without coefficients into a value
            }
        }*/

        eprintln!("Normalized system: {:?}", self);
    }

    pub fn remap(mut self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) -> Self {
        for relation in &mut self.relations {
            relation.combination = relation.combination.clone().remap(old_to_new);
        }

        self
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.relations
            .iter()
            .flat_map(|relation| relation.combination.monomials.keys().copied())
            .collect()
    }
}

impl Join for LinearBitvector {
    fn join(self, other: &Self) -> Self {
        assert_eq!(self.bound(), other.bound());

        // single-layer lattice
        if &self == other {
            self
        } else {
            Self::Top(self.bound())
        }
    }
}

impl Debug for LinearBitvector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LinearBitvector::Top(bound) => write!(f, "⊤({})", bound.width()),
            LinearBitvector::Combination(combination) => Debug::fmt(combination, f),
            LinearBitvector::System(system) => Debug::fmt(system, f),
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

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.combination, f)?;

        let op = if self.slack.is_zero() { "==" } else { "<=" };

        write!(f, " {} {}", op, self.slack)
    }
}

impl Debug for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for relation in &self.relations {
            if is_first {
                is_first = false;
            } else if self.universal {
                write!(f, " ∧ ")?;
            } else {
                write!(f, " ∨ ")?;
            }

            Debug::fmt(relation, f)?;
        }
        Ok(())
    }
}
