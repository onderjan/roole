use super::SymbolicDomain;
use crate::domain::bitvector::{RBound, abstr::BitvectorDomain, concr::ConcreteBitvector};

impl BitvectorDomain for SymbolicDomain {
    type Bound = RBound;

    fn bound(&self) -> RBound {
        match &self {
            SymbolicDomain::Top(bound) => *bound,
            SymbolicDomain::Linear(linear) => linear.bound(),
        }
    }

    fn single_value(value: ConcreteBitvector<RBound>) -> Self {
        SymbolicDomain::from_concrete(value)
    }

    fn top(bound: RBound) -> Self {
        SymbolicDomain::Top(bound)
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        None
    }
}
