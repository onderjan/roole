use crate::{
    domain::bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
    problem::{domain::OperationDomain, linear::LinearCombination},
};

impl BitvectorDomain for OperationDomain {
    type Bound = RBound;

    fn bound(&self) -> RBound {
        match &self {
            OperationDomain::Top(bound) => *bound,
            OperationDomain::Combination(combination) => combination.bound(),
            OperationDomain::System(_) => RBound::single_bit_bound(),
        }
    }

    fn single_value(value: ConcreteBitvector<RBound>) -> Self {
        OperationDomain::Combination(LinearCombination::from_constant(value))
    }

    fn top(bound: RBound) -> Self {
        OperationDomain::Top(bound)
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        None
    }
}
