mod combination;
mod relation;
mod support;

pub use {combination::LinearCombination, relation::LinearRelation};

use serde::{Deserialize, Serialize};
use vec1::Vec1;

/// A system of linear relations.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    /// If true, the system is a conjunction of relations. If false, it is a disjunction.
    pub universal: bool,
    /// Linear relations.
    pub relations: Vec1<LinearRelation>,
}
