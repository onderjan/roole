mod combination;
mod relation;
mod system;

use std::fmt::Debug;

use crate::{domain::bitvector::BitvectorBound, problem::formula::FormulaId};
use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};

pub use {combination::LinearCombination, relation::LinearRelation, system::LinearSystem};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearOperation {
    Combination(LinearCombination),
    System(LinearSystem),
}

impl LinearOperation {
    pub fn result_width(&self) -> u32 {
        match self {
            LinearOperation::Combination(combination) => combination.constant.bound().width(),
            LinearOperation::System(_) => 1,
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
}

impl Debug for LinearOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Combination(combination) => Debug::fmt(combination, f),
            Self::System(system) => Debug::fmt(system, f),
        }
    }
}
