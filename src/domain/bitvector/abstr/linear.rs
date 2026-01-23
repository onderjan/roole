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
enum LinearRelation {
    Eq(LinearCombination),
    Ne(LinearCombination),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    universal: bool,
    relations: Vec<LinearRelation>,
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
