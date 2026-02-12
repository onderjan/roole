use std::fmt::{Debug, UpperHex};

use serde::{Deserialize, Serialize};

use super::LinearSlice;
use crate::domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearMonomial {
    pub coefficient: ConcreteBitvector<RBound>,
    pub slice: LinearSlice,
}

impl LinearMonomial {
    pub(super) fn new(coefficient: ConcreteBitvector<RBound>, slice: LinearSlice) -> Self {
        Self { coefficient, slice }
    }

    pub fn bound(&self) -> RBound {
        self.coefficient.bound()
    }

    pub fn overflows(&self) -> bool {
        // the width needed to represent the monomial product is
        // sum of coefficient width and slice width minus 1
        // this is precise

        let coeff_width = self.coefficient.num_needed_bits();
        let slice_width = self.slice.width.get();
        let product_width = coeff_width + slice_width - 1;

        let bound_width = self.coefficient.bound().width();

        product_width > bound_width
    }

    pub(super) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        if !self.coefficient.is_one() {
            if hex {
                write!(f, "{:#X}*", self.coefficient)?;
            } else {
                write!(f, "{:?}*", self.coefficient)?;
            }
        }

        write!(f, "{:?} mod ", self.slice)?;
        if hex {
            write!(f, "{:#X}", 1u128 << self.bound().width())
        } else {
            write!(f, "{:?}", 1u128 << self.bound().width())
        }
    }
}

impl Debug for LinearMonomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for LinearMonomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
