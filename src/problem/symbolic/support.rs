use std::fmt::{Debug, UpperHex};

use super::{SymbolicDomain, linear::LinearSystem};
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

    pub(super) fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        let SymbolicDomain::Linear(linear) = self else {
            return None;
        };

        linear.constant_value()
    }

    pub fn from_concrete(constant: ConcreteBitvector<RBound>) -> Self {
        Self::Linear(LinearSystem::from_concrete(constant))
    }

    pub fn from_bool(value: bool) -> Self {
        Self::Linear(LinearSystem::from_bool(value))
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        Self::Linear(LinearSystem::from_formula(formula_id, bound))
    }

    pub(super) fn unary_op(self, linear_fn: impl Fn(LinearSystem) -> LinearSystem) -> Self {
        let bound = self.bound();
        let SymbolicDomain::Linear(system) = self else {
            return Self::Top(bound);
        };

        SymbolicDomain::Linear((linear_fn)(system))
    }

    pub(super) fn unary_op_try<E>(
        self,
        linear_fn: impl Fn(LinearSystem) -> Result<LinearSystem, E>,
    ) -> Self {
        let bound = self.bound();
        let SymbolicDomain::Linear(system) = self else {
            return Self::Top(bound);
        };

        match (linear_fn)(system) {
            Ok(system) => Self::Linear(system),
            Err(_) => Self::Top(bound),
        }
    }

    pub(super) fn binary_op_try<E>(
        self,
        rhs: Self,
        linear_fn: impl Fn(LinearSystem, LinearSystem) -> Result<LinearSystem, E>,
        single_bit_result: bool,
    ) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        let (SymbolicDomain::Linear(lhs), SymbolicDomain::Linear(rhs)) = (self, rhs) else {
            return Self::Top(bound);
        };

        match (linear_fn)(lhs, rhs) {
            Ok(system) => Self::Linear(system),
            Err(_) => {
                let result_bound = if single_bit_result {
                    RBound::single_bit_bound()
                } else {
                    bound
                };
                Self::Top(result_bound)
            }
        }
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
