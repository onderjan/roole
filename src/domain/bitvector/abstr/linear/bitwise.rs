use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, linear::LinearBitvector},
        concr::ConcreteBitvector,
    },
    traits::forward::{Bitwise, HwArith},
};

impl<B: BitvectorBound> Bitwise for LinearBitvector<B> {
    fn bit_not(self) -> Self {
        // bit_not(x) = arith_neg(x) - 1

        let mut arith_neg = self.arith_neg();

        let Some(combination) = &mut arith_neg.combination else {
            // already top value
            return arith_neg;
        };

        combination.constant = combination
            .constant
            .sub(ConcreteBitvector::one(arith_neg.bound));

        combination.normalize();

        arith_neg
    }
    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        // TODO: handle masking situations

        LinearBitvector::top(self.bound)
    }
    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        // TODO: handle masking situations

        LinearBitvector::top(self.bound)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        // TODO: handle masking situations

        LinearBitvector::top(self.bound)
    }
}
