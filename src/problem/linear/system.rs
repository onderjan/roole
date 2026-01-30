use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use vec1::Vec1;

use crate::problem::{formula::FormulaId, linear::LinearRelation};

mod ops;

/// A system of linear relations.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    /// If true, the system is a conjunction of relations. If false, it is a disjunction.
    pub universal: bool,
    /// Linear relations.
    pub relations: Vec1<LinearRelation>,
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

    pub fn remap(&mut self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) {
        for relation in &mut self.relations {
            relation.combination.remap(old_to_new);
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.relations
            .iter()
            .flat_map(|relation| relation.combination.monomials.keys().copied())
            .collect()
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
