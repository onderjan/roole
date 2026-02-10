use std::fmt::{Debug, UpperHex};

use super::{
    SymbolicDomain,
    linear::{LinearExpression, LinearPolynomial, LinearSystem},
};
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::Join,
    },
    problem::formula::FormulaId,
};

impl SymbolicDomain {
    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self {
            SymbolicDomain::Top(_) => vec![],
            SymbolicDomain::Linear(linear) => linear.used_ids(),
        }
    }

    pub(super) fn try_into_polynomial(self) -> Result<LinearPolynomial, SymbolicDomain> {
        let SymbolicDomain::Linear(linear) = self else {
            return Err(self);
        };

        match linear.try_into_polynomial() {
            Ok(polynomial) => Ok(polynomial),
            Err(linear) => Err(Self::Linear(linear)),
        }
    }

    pub(super) fn try_into_expression(self) -> Result<LinearExpression, SymbolicDomain> {
        let SymbolicDomain::Linear(linear) = self else {
            return Err(self);
        };

        match linear.try_into_expression() {
            Ok(expression) => Ok(expression),
            Err(linear) => Err(Self::Linear(linear)),
        }
    }

    pub(super) fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        let SymbolicDomain::Linear(linear) = self else {
            return None;
        };

        linear.constant_value()
    }

    pub fn from_expression(expression: LinearExpression) -> Self {
        Self::Linear(LinearSystem::from_expression(expression))
    }

    pub fn from_polynomial(polynomial: LinearPolynomial) -> Self {
        Self::Linear(LinearSystem::from_polynomial(polynomial))
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        Self::from_polynomial(LinearPolynomial::from_formula(formula_id, bound))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        match &self {
            SymbolicDomain::Top(bound) => write!(f, "⊤({})", bound.width()),
            SymbolicDomain::Linear(linear) => linear.format(f, hex),
        }
    }
}

impl Join for SymbolicDomain {
    fn join(self, other: &Self) -> Self {
        assert_eq!(self.bound(), other.bound());

        // single-layer lattice
        if &self == other {
            self
        } else {
            Self::Top(self.bound())
        }
    }
}

impl Debug for SymbolicDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for SymbolicDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
