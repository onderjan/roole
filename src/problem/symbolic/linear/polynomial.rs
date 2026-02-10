use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{LinearMonomial, LinearSlice};
use crate::domain::bitvector::{RBound, concr::ConcreteBitvector};

mod arith;
mod bitwise;
mod constant;
mod eq;
mod eval;
mod ext;
mod format;
mod misc;
mod shift;
mod support;

/// A linear combination of bitvectors and a constant.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearPolynomial {
    linear_terms: BTreeMap<LinearSlice, ConcreteBitvector<RBound>>,
    constant_term: ConcreteBitvector<RBound>,
}
