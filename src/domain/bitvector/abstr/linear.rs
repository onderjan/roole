mod domain;
mod support;

use std::{collections::BTreeMap, num::NonZeroI64};

use serde::{Deserialize, Serialize};

use crate::{domain::bitvector::BitvectorBound, problem::formula::FormulaId};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearCombination {
    constant: i64,
    variables: BTreeMap<FormulaId, NonZeroI64>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearBitvector<B: BitvectorBound> {
    bound: B,
    combination: Option<LinearCombination>,
}
