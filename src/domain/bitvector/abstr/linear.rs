mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;

mod domain;
mod support;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::formula::FormulaId,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    constant: ConcreteBitvector<RBound>,
    coefficients: BTreeMap<FormulaId, ConcreteBitvector<RBound>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearEquation {
    side: LinearCombination,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    equations: Vec<LinearEquation>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum LinearType {
    Top,
    Combination(LinearCombination),
    System(LinearSystem),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearBitvector {
    bound: RBound,
    ty: LinearType,
}
