mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;

mod bitvector;
mod linear;
mod support;

use serde::{Deserialize, Serialize};

use crate::domain::bitvector::RBound;

pub use linear::LinearSystem;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationDomain {
    Top(RBound),
    Linear(LinearSystem),
}
