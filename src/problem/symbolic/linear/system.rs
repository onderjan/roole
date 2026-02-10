use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::formula::FormulaId,
};
use serde::{Deserialize, Serialize};

use super::{LinearExpression, LinearPolynomial};

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod eval;
mod ext;
mod shift;
mod support;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    conjunction: bool,
    bound: RBound,
    expressions: Vec<LinearExpression>,
}

impl LinearSystem {
    pub fn bound(&self) -> RBound {
        self.bound
    }

    pub fn from_concrete(constant: ConcreteBitvector<RBound>) -> Self {
        Self::from_polynomial(LinearPolynomial::from_concrete(constant))
    }

    pub fn from_bool(value: bool) -> Self {
        Self::from_polynomial(LinearPolynomial::from_bool(value))
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        Self::from_polynomial(LinearPolynomial::from_formula(formula_id, bound))
    }
}
