use crate::{
    domain::{bitvector::concr::ConcreteBitvector, traits::forward::HwArith},
    problem::linear::LinearCombination,
};

impl LinearCombination {
    pub fn bit_not(self) -> Self {
        let mut result = self.arith_neg();
        result.constant = result.constant.sub(ConcreteBitvector::one(result.bound()));
        result.normalize();
        result
    }
}
