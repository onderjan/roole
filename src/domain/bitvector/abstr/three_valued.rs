#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ThreeValuedBitvector<B: BitvectorBound> {
    zeros: ConcreteBitvector<B>,
    ones: ConcreteBitvector<B>,
}

use serde::{Deserialize, Serialize};

use crate::domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector};

pub struct InvalidZerosOnes;
