use std::fmt::Debug;
use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use crate::{
    domain::bitvector::{BitvectorBound, RBound},
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

    pub fn contains(&self, contained: &Self) -> bool {
        if contained.lsb < self.lsb {
            return false;
        }
        if contained.lsb + contained.width.get() > self.lsb + self.width.get() {
            return false;
        }
        true
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
