use std::collections::BTreeMap;

use crate::domain::bitvector::{
    BitvectorBound,
    abstr::{
        BitvectorDisplay, BitvectorDomain,
        linear::{LinearBitvector, LinearCombination},
    },
    concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector},
};

impl<B: BitvectorBound> BitvectorDomain for LinearBitvector<B> {
    type Bound = B;

    type General<X: BitvectorBound> = LinearBitvector<X>;

    fn bound(&self) -> Self::Bound {
        self.bound
    }

    fn single_value(value: ConcreteBitvector<Self::Bound>) -> Self {
        Self {
            bound: value.bound(),
            combination: Some(LinearCombination {
                constant: value,
                coefficients: BTreeMap::new(),
            }),
        }
    }

    fn top(bound: Self::Bound) -> Self {
        Self {
            bound,
            combination: None,
        }
    }

    fn meet(self, other: &Self) -> Option<Self> {
        todo!()
    }

    fn umin(&self) -> UnsignedBitvector<Self::Bound> {
        todo!()
    }

    fn umax(&self) -> UnsignedBitvector<Self::Bound> {
        todo!()
    }

    fn smin(&self) -> SignedBitvector<Self::Bound> {
        todo!()
    }

    fn smax(&self) -> SignedBitvector<Self::Bound> {
        todo!()
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>> {
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
