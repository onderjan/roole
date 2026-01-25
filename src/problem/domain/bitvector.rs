use std::collections::BTreeMap;

use crate::{
    domain::bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
    problem::domain::{LinearBitvector, LinearCombination},
};

impl BitvectorDomain for LinearBitvector {
    type Bound = RBound;

    fn bound(&self) -> RBound {
        match &self {
            LinearBitvector::Top(bound) => *bound,
            LinearBitvector::Combination(combination) => combination.bound(),
            LinearBitvector::System(_) => RBound::single_bit_bound(),
        }
    }

    fn single_value(value: ConcreteBitvector<RBound>) -> Self {
        LinearBitvector::Combination(LinearCombination {
            constant: value,
            coefficients: BTreeMap::new(),
        })
    }

    fn top(bound: RBound) -> Self {
        LinearBitvector::Top(bound)
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
        None
    }
}
