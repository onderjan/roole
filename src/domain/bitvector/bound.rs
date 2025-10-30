use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

pub trait BitvectorBound: Clone + Copy + PartialEq + Eq + Hash + Debug {
    type SingleBit: BitvectorBound<SingleBit = Self::SingleBit>;

    fn width(&self) -> u32;
    fn mask(&self) -> u64;
    fn sign_bit_mask(&self) -> u64;

    fn allowed(&self, value: u64) -> bool {
        value <= self.mask()
    }

    fn single_bit_bound() -> Self::SingleBit;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct RBound {
    width: u32,
}

impl RBound {
    pub fn new(width: u32) -> Self {
        Self { width }
    }
}

impl BitvectorBound for RBound {
    type SingleBit = RBound;

    fn width(&self) -> u32 {
        self.width
    }

    fn mask(&self) -> u64 {
        compute_u64_mask(self.width)
    }

    fn sign_bit_mask(&self) -> u64 {
        compute_u64_sign_bit_mask(self.width)
    }

    fn single_bit_bound() -> Self::SingleBit {
        RBound { width: 1 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct CBound<const W: u32>;

impl<const W: u32> BitvectorBound for CBound<W> {
    type SingleBit = CBound<1>;

    fn width(&self) -> u32 {
        W
    }
    fn mask(&self) -> u64 {
        compute_u64_mask(W)
    }

    fn sign_bit_mask(&self) -> u64 {
        compute_u64_sign_bit_mask(W)
    }

    fn single_bit_bound() -> Self::SingleBit {
        CBound::<1>
    }
}

pub const fn compute_u64_mask(width: u32) -> u64 {
    if width == 0 {
        return 0;
    }
    if width == u64::BITS {
        // this would fail in checked shl,
        // but the mask is just full of ones
        return 0u64.wrapping_sub(1u64);
    }
    let num_values = u64::checked_shl(1u64, width);
    if let Some(num_values) = num_values {
        num_values.wrapping_sub(1u64)
    } else {
        panic!("Bit mask length should fit");
    }
}

const fn compute_u64_sign_bit_mask(width: u32) -> u64 {
    if width == 0 {
        return 0;
    }
    // the highest bit within mask (unless length is 0)
    let result = 1u64.checked_shl(width - 1);
    if let Some(result) = result {
        result
    } else {
        panic!("Sign bit mask length should fit")
    }
}
