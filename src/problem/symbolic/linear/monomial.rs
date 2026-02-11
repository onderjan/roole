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

    pub fn might_overflow(&self) -> bool {
        let coefficient = self.coefficient.to_u64();
        let slice_width = self.slice.width.get();

        let Some(above_max_value) = coefficient.checked_shl(slice_width) else {
            return true;
        };

        let Some(max_value) = above_max_value.checked_sub(1) else {
            return true;
        };

        let max_value_allowed = self.coefficient.bound().allowed(max_value);

        !max_value_allowed
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
