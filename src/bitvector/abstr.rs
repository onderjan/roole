use std::ops::{BitAnd, BitOr, BitXor, Not};

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

trait Primitive:
    Clone
    + Copy
    + num::Unsigned
    + Not<Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
{
    type Signed: Clone + Copy + PartialEq + Eq + PartialOrd + Ord;

    fn width_mask(width: u32) -> Self;
    fn sign_bit_mask(width: u32) -> Self;
    fn cast_signed(self, width: u32) -> Self::Signed;
}

#[derive(Clone, Copy, Hash)]
pub struct ThreeValued<T: Primitive> {
    zeros: T,
    ones: T,
}
