use crate::{
    domain::bitvector::{RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
    problem::{domain::OperationDomain, operation::LinearPolynomial},
};

impl BitvectorDomain for OperationDomain {
    type Bound = RBound;

    fn bound(&self) -> RBound {
        match &self {
            OperationDomain::Top(bound) => *bound,
            OperationDomain::Linear(linear) => linear.result_bound(),
        }
    }

    fn single_value(value: ConcreteBitvector<RBound>) -> Self {
        OperationDomain::from_polynomial(LinearPolynomial::from_constant(value))
    }

    fn top(bound: RBound) -> Self {
        OperationDomain::Top(bound)
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        None
    }
}
