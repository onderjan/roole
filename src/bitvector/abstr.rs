use crate::bitvector::concr::RUnsigned;

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

#[derive(Clone, Copy, Hash, Debug)]
pub struct ThreeValued<U: RUnsigned> {
    zeros: U,
    ones: U,
}
