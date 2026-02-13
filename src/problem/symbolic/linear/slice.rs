use std::fmt::Debug;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::{Bitwise, HwArith},
    },
    problem::formula::FormulaId,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct LinearSlice {
    pub(super) formula_id: FormulaId,
    pub(super) lsb: u32,
    pub(super) width: NonZeroU32,
}

impl LinearSlice {
    fn new(formula_id: FormulaId, lsb: u32, width: NonZeroU32) -> Self {
        Self {
            formula_id,
            lsb,
            width,
        }
    }

    pub(super) fn from_bounded(formula_id: FormulaId, bound: RBound) -> Option<Self> {
        NonZeroU32::new(bound.width()).map(|width| Self::new(formula_id, 0, width))
    }

    pub(super) fn from_mask(formula_id: FormulaId, mask: ConcreteBitvector<RBound>) -> Self {
        // the mask must have continguous ones
        let turned_off_rightmost_ones = mask.bit_and(mask.bit_not()).add(mask).bit_and(mask);
        assert!(turned_off_rightmost_ones.is_nonzero());

        let lsb = mask.to_u64().trailing_zeros();
        let width = mask.to_u64().count_ones();

        Self {
            formula_id,
            lsb,
            width: NonZeroU32::new(width).expect("Width should be nonzero"),
        }
    }

    pub fn contains(&self, contained: &Self) -> bool {
        // must have the same formula id
        if self.formula_id != contained.formula_id {
            return false;
        }

        // contained must be within self
        contained.lsb >= self.lsb && contained.above_msb() <= self.above_msb()
    }

    // Mask applied to the slice formula.
    pub fn formula_mask(&self, bound: RBound) -> ConcreteBitvector<RBound> {
        let including_below_lsb = ConcreteBitvector::from_ones_width(self.above_msb(), bound);
        let below_lsb = ConcreteBitvector::from_ones_width(self.lsb, bound);
        including_below_lsb.sub(below_lsb)
    }

    // Output mask. Starts from the lowest significant bit and has the number of consecutive ones equal to width.
    pub fn output_mask(&self, bound: RBound) -> ConcreteBitvector<RBound> {
        ConcreteBitvector::from_ones_width(self.width.get(), bound)
    }

    pub(super) fn above_msb(&self) -> u32 {
        self.lsb + self.width.get()
    }
}

impl Debug for LinearSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.width.get() == 1 {
            write!(f, "{:?}[{}]", self.formula_id, self.lsb,)
        } else {
            write!(
                f,
                "{:?}[{}..{}]",
                self.formula_id,
                self.lsb,
                self.lsb + self.width.get()
            )
        }
    }
}
