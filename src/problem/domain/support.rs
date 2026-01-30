use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::Join,
    },
    problem::{domain::OperationDomain, formula::FormulaId, linear::LinearCombination},
};

impl OperationDomain {
    pub fn for_formula_id(formula_id: FormulaId, bound: RBound) -> Self {
        let constant = ConcreteBitvector::zero(bound);
        let monomials = BTreeMap::from_iter([(formula_id, ConcreteBitvector::one(bound))]);

        OperationDomain::Combination(LinearCombination {
            constant,
            monomials,
        })
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self {
            OperationDomain::Top(_) => vec![],
            OperationDomain::Combination(combination) => combination.used_ids(),
            OperationDomain::System(system) => system.used_ids(),
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
        match &self {
            OperationDomain::Top(bound) => write!(f, "⊤({})", bound.width()),
            OperationDomain::Combination(combination) => Debug::fmt(combination, f),
            OperationDomain::System(system) => Debug::fmt(system, f),
        }
    }
}
