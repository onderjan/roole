use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::{
        eval::EvaluableDomain,
        formula::FormulaId,
        operation::{LinearCombination, linear::LinearRelation},
    },
};

mod ops;

/// A system of linear relations.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearSystem {
    Single(LinearRelation),
    Conjunction(Vec<LinearRelation>),
    Disjunction(Vec<LinearRelation>),
}

impl LinearSystem {
    pub fn from_eq(lhs: LinearCombination, rhs: LinearCombination) -> Self {
        // if both are combinations, make into an equality

        let combination = lhs.sub(rhs);
        let slack = ConcreteBitvector::zero(combination.bound());
        LinearSystem::Single(LinearRelation::new(combination, slack))
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            LinearSystem::Single(relation) => relation.evaluate(fetch),
            LinearSystem::Conjunction(relations) => {
                Self::evaluate_relations(fetch, relations, true)
            }
            LinearSystem::Disjunction(relations) => {
                Self::evaluate_relations(fetch, relations, false)
            }
        }
    }

    pub fn evaluate_relations<D: EvaluableDomain>(
        fetch: impl Fn(FormulaId) -> D,
        relations: &[LinearRelation],
        universal: bool,
    ) -> D {
        let bound = RBound::new(1);
        let mut result = if universal {
            // start with 1
            D::single_value(ConcreteBitvector::one(bound))
        } else {
            // start with 0
            D::single_value(ConcreteBitvector::zero(bound))
        };

        for relation in relations {
            let relation_result = relation.evaluate(&fetch);
            if universal {
                result = result.bit_and(relation_result);
            } else {
                result = result.bit_or(relation_result);
            }
        }
        result
    }

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
        let relations = match self {
            LinearSystem::Single(relation) => {
                relation.remap(old_to_new);
                return;
            }
            LinearSystem::Conjunction(relations) => relations,
            LinearSystem::Disjunction(relations) => relations,
        };

        for relation in relations {
            relation.remap(old_to_new);
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.relations_iter()
            .flat_map(|relation| relation.used_ids())
            .collect()
    }

    fn relations_iter(&self) -> Box<dyn Iterator<Item = &LinearRelation> + '_> {
        match self {
            LinearSystem::Single(relation) => Box::new(std::iter::once(relation)),
            LinearSystem::Conjunction(relations) => Box::new(relations.iter()),
            LinearSystem::Disjunction(relations) => Box::new(relations.iter()),
        }
    }

    fn into_relations_iter(self) -> Box<dyn Iterator<Item = LinearRelation>> {
        match self {
            LinearSystem::Single(relation) => Box::new(std::iter::once(relation)),
            LinearSystem::Conjunction(relations) => Box::new(relations.into_iter()),
            LinearSystem::Disjunction(relations) => Box::new(relations.into_iter()),
        }
    }
}

impl Debug for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let universal = matches!(self, LinearSystem::Conjunction(_));
        let mut is_first = true;
        for relation in self.relations_iter() {
            if is_first {
                is_first = false;
            } else if universal {
                write!(f, " ∧ ")?;
            } else {
                write!(f, " ∨ ")?;
            }

            Debug::fmt(relation, f)?;
        }
        Ok(())
    }
}
