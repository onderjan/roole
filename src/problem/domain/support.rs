use std::fmt::{Debug, UpperHex};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::Join,
    },
    problem::{
        domain::OperationDomain,
        formula::FormulaId,
        operation::{LinearPolynomial, LinearSystem},
    },
};

impl OperationDomain {
    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self {
            OperationDomain::Top(_) => vec![],
            OperationDomain::Linear(linear) => linear.used_ids(),
        }
    }

    pub(super) fn try_polynomial(self) -> Result<LinearPolynomial, OperationDomain> {
        let OperationDomain::Linear(linear) = self else {
            return Err(self);
        };

        match linear.try_into_polynomial() {
            Ok(polynomial) => Ok(polynomial),
            Err(linear) => Err(Self::Linear(linear)),
        }
    }

    pub(super) fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        let OperationDomain::Linear(linear) = self else {
            return None;
        };

        linear.constant_value()
    }

    pub fn from_polynomial(polynomial: LinearPolynomial) -> Self {
        Self::Linear(LinearSystem::from_polynomial(polynomial))
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        match &self {
            OperationDomain::Top(bound) => write!(f, "⊤({})", bound.width()),
            OperationDomain::Linear(linear) => linear.format(f, hex),
        }
    }
}

impl Join for OperationDomain {
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

impl Debug for OperationDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for OperationDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
