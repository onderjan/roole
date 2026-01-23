use crate::domain::{
    bitvector::{
        abstr::{
            BitvectorDomain,
            linear::{LinearBitvector, LinearType},
        },
        concr::ConcreteBitvector,
    },
    traits::forward::{Bitwise, HwArith},
};

impl Bitwise for LinearBitvector {
    fn bit_not(self) -> Self {
        // bit_not(x) = arith_neg(x) - 1

        let mut arith_neg = self.arith_neg();

        let LinearType::Combination(combination) = &mut arith_neg.ty else {
            return Self::top(arith_neg.bound);
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
