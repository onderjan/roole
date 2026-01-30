mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;

mod bitvector;
mod support;

use serde::{Deserialize, Serialize};

use crate::{
    domain::bitvector::RBound,
    problem::linear::{LinearCombination, LinearSystem},
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationDomain {
    Top(RBound),
    Combination(LinearCombination),
    System(LinearSystem),
}
