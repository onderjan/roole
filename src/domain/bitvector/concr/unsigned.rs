use std::{
    fmt::{Debug, Display},
    ops::{Add, BitAnd, BitOr, BitXor, Mul, Not, Shl, Shr, Sub},
};

use serde::{Deserialize, Serialize};

use crate::domain::{
    bitvector::BitvectorBound,
    traits::forward::{BExt, Bitwise, HwArith, HwShift},
};

use super::ConcreteBitvector;

#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsignedBitvector<B: BitvectorBound>(ConcreteBitvector<B>);

impl<B: BitvectorBound> UnsignedBitvector<B> {
    pub fn bound(&self) -> B {
        self.0.bound
    }

    pub fn new_zero(bound: B) -> Self {
        UnsignedBitvector(ConcreteBitvector::new_zero(bound))
    }

    pub fn new_one(bound: B) -> Self {
        UnsignedBitvector(ConcreteBitvector::new_one(bound))
    }

    pub(super) const fn from_bitvector(bitvector: ConcreteBitvector<B>) -> Self {
        UnsignedBitvector(bitvector)
    }

    pub fn cast_bitvector(self) -> ConcreteBitvector<B> {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_nonzero(&self) -> bool {
        self.0.is_nonzero()
    }

    pub fn ext<X: BitvectorBound>(self, new_bound: X) -> UnsignedBitvector<X> {
        UnsignedBitvector(self.0.uext(new_bound))
    }

    pub fn div_wrapping_or_full(self, rhs: Self) -> Self {
        self.cast_bitvector()
            .udiv_wrapping_or_all_ones(rhs.cast_bitvector())
            .into_unsigned()
    }

    pub fn rem_wrapping_or_dividend(self, rhs: Self) -> Self {
        self.cast_bitvector()
            .urem_wrapping_or_dividend(rhs.cast_bitvector())
            .into_unsigned()
    }
}

impl<B: BitvectorBound> Add<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn add(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl<B: BitvectorBound> Sub<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn sub(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl<B: BitvectorBound> Mul<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn mul(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl<B: BitvectorBound> Not for UnsignedBitvector<B> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.bit_not())
    }
}

impl<B: BitvectorBound> BitAnd for UnsignedBitvector<B> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_and(rhs.0))
    }
}

impl<B: BitvectorBound> BitOr for UnsignedBitvector<B> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_or(rhs.0))
    }
}

impl<B: BitvectorBound> BitXor for UnsignedBitvector<B> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0.bit_xor(rhs.0))
    }
}

impl<B: BitvectorBound> Shl<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn shl(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        // both signed and unsigned use logic shift left
        Self(self.0.logic_shl(rhs.0))
    }
}

impl<B: BitvectorBound> Shr<UnsignedBitvector<B>> for UnsignedBitvector<B> {
    type Output = Self;

    fn shr(self, rhs: UnsignedBitvector<B>) -> Self::Output {
        // signed uses arithmetic shift right
        Self(self.0.logic_shr(rhs.0))
    }
}

impl<B: BitvectorBound> PartialOrd for UnsignedBitvector<B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: BitvectorBound> Ord for UnsignedBitvector<B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // unsigned comparison
        self.0.unsigned_cmp(&other.0)
    }
}

impl<B: BitvectorBound> Debug for UnsignedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // defer to bitvector
        Debug::fmt(&self, f)
    }
}

impl<B: BitvectorBound> Display for UnsignedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // defer to bitvector
        Display::fmt(&self, f)
    }
}
