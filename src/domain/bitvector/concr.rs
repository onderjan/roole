#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

mod signed;
mod unsigned;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConcreteBitvector<B: BitvectorBound> {
    bound: B,
    value: u64,
}

pub use signed::SignedBitvector;
pub use unsigned::UnsignedBitvector;

use crate::domain::bitvector::BitvectorBound;

#[derive(Clone, Copy)]
pub struct OutsideBound<T> {
    width: u32,
    value: T,
    min_value: T,
    max_value: T,
}
