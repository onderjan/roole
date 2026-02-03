use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
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
        // TODO: normalize system with slack
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
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

    pub fn try_into_combination(self) -> Result<LinearCombination, Self> {
        let LinearSystem::Single(relation) = self else {
            return Err(self);
        };

        let bound = relation.combination().bound();

        match bound.width() {
            0 => {
                // can convert into empty combination
                Ok(LinearCombination::empty(bound))
            }
            1 => {
                // can convert into Boolean
                if relation.slack().is_nonzero() {
                    // this is always true
                    return Ok(LinearCombination::single_bit(true));
                }

                // the relation is left <= 0, i.e. left == 0
                // we must bit-not to obtain (!left) == (!1)
                // i.e. !left == 1, which can be converted to combination !left

                let bit_not_combination = relation.into_combination().bit_not();

                Ok(bit_not_combination)
            }
            _ => {
                // cannot convert
                Err(LinearSystem::Single(relation))
            }
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
