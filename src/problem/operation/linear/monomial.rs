use serde::{Deserialize, Serialize};

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::operation::linear::slice::LinearSlice,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearMonomial {
    pub coefficient: ConcreteBitvector<RBound>,
    pub slice: LinearSlice,
}

impl LinearMonomial {
    pub(super) fn new(coefficient: ConcreteBitvector<RBound>, slice: LinearSlice) -> Self {
        Self { coefficient, slice }
    }
}
