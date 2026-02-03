mod combination;
mod relation;
mod slice;
mod system;

use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::bitvector::{BitvectorBound, RBound},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};
use serde::{Deserialize, Serialize};

pub use {combination::LinearCombination, relation::LinearRelation, system::LinearSystem};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearOperation(LinearOperationType);

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum LinearOperationType {
    Combination(LinearCombination),
    System(LinearSystem),
}

impl LinearOperation {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match &self.0 {
            LinearOperationType::Combination(combination) => combination.evaluate(fetch),
            LinearOperationType::System(system) => system.evaluate(fetch),
        }
    }

    pub fn from_combination(combination: LinearCombination) -> Self {
        Self(LinearOperationType::Combination(combination))
    }

    pub fn from_system(system: LinearSystem) -> Self {
        Self(LinearOperationType::System(system))
    }

    pub fn try_into_combination(self) -> Result<LinearCombination, LinearOperation> {
        match self.0 {
            LinearOperationType::Combination(combination) => Ok(combination),
            ty => Err(Self(ty)),
        }
    }

    pub fn try_into_system(self) -> Result<LinearSystem, LinearOperation> {
        match self.0 {
            LinearOperationType::System(system) => Ok(system),
            ty => Err(Self(ty)),
        }
    }

    pub fn result_bound(&self) -> RBound {
        match &self.0 {
            LinearOperationType::Combination(combination) => combination.bound(),
            LinearOperationType::System(_) => RBound::single_bit_bound(),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self.0 {
            LinearOperationType::Combination(combination) => combination.used_ids(),
            LinearOperationType::System(system) => system.used_ids(),
        }
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        match &mut self.0 {
            LinearOperationType::Combination(combination) => combination.remap(old_to_new),
            LinearOperationType::System(system) => system.remap(old_to_new),
        }
    }

    pub fn bit_not(self) -> Self {
        match self.0 {
            LinearOperationType::Combination(combination) => {
                Self(LinearOperationType::Combination(combination.bit_not()))
            }
            LinearOperationType::System(system) => match system.bit_not() {
                Ok(system) => Self::from_system(system),
                Err(constant) => Self(LinearOperationType::Combination(
                    LinearCombination::single_bit(constant),
                )),
            },
        }
    }
}

impl Debug for LinearOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            LinearOperationType::Combination(combination) => Debug::fmt(combination, f),
            LinearOperationType::System(system) => Debug::fmt(system, f),
        }
    }
}
