use std::collections::BTreeMap;

use crate::{
    domain::bitvector::{
        BitvectorBound, RBound,
        abstr::{
            BitvectorDisplay, BitvectorDomain,
            linear::{LinearBitvector, LinearCombination},
        },
        concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector},
    },
    problem::formula::FormulaId,
};

impl BitvectorDomain for LinearBitvector {
    type Bound = RBound;

    fn bound(&self) -> RBound {
        match &self {
            LinearBitvector::Top(bound) => *bound,
            LinearBitvector::Combination(combination) => combination.bound(),
            LinearBitvector::System(system) => RBound::single_bit_bound(),
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

    fn formula(bound: RBound, formula: FormulaId) -> Self {
        let mut coefficients = BTreeMap::new();
        coefficients.insert(formula, ConcreteBitvector::one(bound));
        LinearBitvector::Combination(LinearCombination {
            constant: ConcreteBitvector::zero(bound),
            coefficients,
        })
    }

    fn meet(self, other: &Self) -> Option<Self> {
        todo!()
    }

    fn umin(&self) -> UnsignedBitvector<RBound> {
        todo!()
    }

    fn umax(&self) -> UnsignedBitvector<RBound> {
        todo!()
    }

    fn smin(&self) -> SignedBitvector<RBound> {
        todo!()
    }

    fn smax(&self) -> SignedBitvector<RBound> {
        todo!()
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<RBound>> {
        todo!()
    }

    fn display(&self) -> BitvectorDisplay {
        todo!()
    }

    fn get_tracker(&self) -> Option<u32> {
        None
    }

    fn assign_tracker(&mut self, _tracker: Option<u32>) {}
}
