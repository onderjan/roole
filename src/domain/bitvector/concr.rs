#[cfg(test)]
mod tests;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;
mod value;

mod signed;
mod unsigned;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcreteValue {
    Small(u64),
    Big(Box<[u64]>),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConcreteBitvector<B: BitvectorBound> {
    bound: B,
    value: ConcreteValue,
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
