mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;

mod bitvector;
mod support;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vec1::Vec1;

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::formula::FormulaId,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    pub constant: ConcreteBitvector<RBound>,
    pub coefficients: BTreeMap<FormulaId, ConcreteBitvector<RBound>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearRelation {
    pub combination: LinearCombination,
    pub slack: ConcreteBitvector<RBound>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    pub universal: bool,
    pub relations: Vec1<LinearRelation>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearBitvector {
    Top(RBound),
    Combination(LinearCombination),
    System(LinearSystem),
}
