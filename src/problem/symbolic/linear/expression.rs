use std::{
    collections::BTreeMap,
    fmt::{Debug, UpperHex},
};

use serde::{Deserialize, Serialize};

use super::{LinearPolynomial, LinearRelation};
use crate::{
    domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

mod arith;
mod bitwise;
mod cmp;
mod normal;
mod support;

/// Either a linear polynomial or a linear relation.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearExpression {
    Polynomial(LinearPolynomial),
    Relation(LinearRelation),
}

impl LinearExpression {
    pub fn bound(&self) -> RBound {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.bound(),
            LinearExpression::Relation(_) => {
                // relation result bound is always 1
                RBound::single_bit_bound()
            }
        }
    }

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.constant_value(),
            LinearExpression::Relation(_) => None,
        }
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.evaluate(fetch),
            LinearExpression::Relation(relation) => relation.evaluate(fetch),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.used_ids(),
            LinearExpression::Relation(relation) => relation.used_ids(),
        }
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.remap(old_to_new),
            LinearExpression::Relation(relation) => relation.remap(old_to_new),
        }
    }

    pub(super) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.format(f, hex),
            LinearExpression::Relation(relation) => relation.format(f, hex),
        }
    }
}

impl Debug for LinearExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for LinearExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
