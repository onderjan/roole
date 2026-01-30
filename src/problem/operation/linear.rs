mod combination;
mod relation;
mod system;

use std::fmt::Debug;

use crate::{
    domain::bitvector::{BitvectorBound, RBound},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};
use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};

pub use {combination::LinearCombination, relation::LinearRelation, system::LinearSystem};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearOperation {
    Combination(LinearCombination),
    System(LinearSystem),
}

impl LinearOperation {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            LinearOperation::Combination(combination) => combination.evaluate(fetch),
            LinearOperation::System(system) => system.evaluate(fetch),
        }
    }

    pub fn result_bound(&self) -> RBound {
        match self {
            LinearOperation::Combination(combination) => combination.bound(),
            LinearOperation::System(_) => RBound::single_bit_bound(),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match self {
            LinearOperation::Combination(combination) => combination.used_ids(),
            LinearOperation::System(system) => system.used_ids(),
        }
    }

    pub fn remap(&mut self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) {
        match self {
            LinearOperation::Combination(combination) => combination.remap(old_to_new),
            LinearOperation::System(system) => system.remap(old_to_new),
        }
    }

    pub fn bit_not(self) -> Self {
        match self {
            LinearOperation::Combination(combination) => {
                LinearOperation::Combination(combination.bit_not())
            }
            LinearOperation::System(system) => match system.bit_not() {
                Ok(system) => LinearOperation::System(system),
                Err(constant) => {
                    LinearOperation::Combination(LinearCombination::single_bit(constant))
                }
            },
        }
    }
}

impl Debug for LinearOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Combination(combination) => Debug::fmt(combination, f),
            Self::System(system) => Debug::fmt(system, f),
        }
    }
}
