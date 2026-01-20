mod domain;
mod support;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::bitvector::BitvectorBound;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LinearId {
    variable: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    constant: i64,
    variables: BTreeMap<LinearId, i64>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearBitvector<B: BitvectorBound> {
    bound: B,
    combination: Option<LinearCombination>,
}
