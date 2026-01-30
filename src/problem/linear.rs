mod combination;
mod support;

pub use combination::LinearCombination;

use serde::{Deserialize, Serialize};
use vec1::Vec1;

use crate::domain::bitvector::{RBound, concr::ConcreteBitvector};

/// A linear relation `combination` <= `slack`.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearRelation {
    /// Left-side linear combination.
    pub combination: LinearCombination,
    /// Right-side slack value. With zero slack, the relation becomes equality.
    pub slack: ConcreteBitvector<RBound>,
}

/// A system of linear relations.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    /// If true, the system is a conjunction of relations. If false, it is a disjunction.
    pub universal: bool,
    /// Linear relations.
    pub relations: Vec1<LinearRelation>,
}
