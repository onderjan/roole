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
    domain::bitvector::{BitvectorBound, concr::ConcreteBitvector},
    problem::formula::FormulaId,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination<B: BitvectorBound> {
    constant: ConcreteBitvector<B>,
    coefficients: BTreeMap<FormulaId, ConcreteBitvector<B>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearBitvector<B: BitvectorBound> {
    bound: B,
    combination: Option<LinearCombination<B>>,
}
