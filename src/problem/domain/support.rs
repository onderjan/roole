use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::Join,
    },
    problem::{
        domain::OperationDomain,
        formula::FormulaId,
        operation::{LinearCombination, LinearOperation, LinearSystem},
    },
};

impl OperationDomain {
    pub fn for_formula_id(formula_id: FormulaId, bound: RBound) -> Self {
        let constant = ConcreteBitvector::zero(bound);
        let monomials = BTreeMap::from_iter([(formula_id, ConcreteBitvector::one(bound))]);

        OperationDomain::from_combination(LinearCombination::new(constant, monomials))
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self {
            OperationDomain::Top(_) => vec![],
            OperationDomain::Linear(linear) => linear.used_ids(),
        }
    }

    pub(super) fn try_combination(self) -> Result<LinearCombination, OperationDomain> {
        if let OperationDomain::Linear(LinearOperation::Combination(combination)) = self {
            Ok(combination)
        } else {
            Err(self)
        }
    }

    pub(super) fn try_system(self) -> Result<LinearSystem, OperationDomain> {
        if let OperationDomain::Linear(LinearOperation::System(system)) = self {
            Ok(system)
        } else {
            Err(self)
        }
    }

    pub fn from_combination(combination: LinearCombination) -> Self {
        Self::Linear(LinearOperation::Combination(combination))
    }

    pub(super) fn from_system(system: LinearSystem) -> Self {
        Self::Linear(LinearOperation::System(system))
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
        match &self {
            OperationDomain::Top(bound) => write!(f, "⊤({})", bound.width()),
            OperationDomain::Linear(linear) => Debug::fmt(linear, f),
        }
    }
}
