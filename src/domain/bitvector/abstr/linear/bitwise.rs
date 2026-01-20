use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, linear::LinearBitvector},
    },
    traits::forward::Bitwise,
};

impl<B: BitvectorBound> Bitwise for LinearBitvector<B> {
    fn bit_not(mut self) -> Self {
        let Some(combination) = &mut self.combination else {
            // already top value
            return self;
        };

        combination.constant = combination.constant.bit_not();

        for coeff in combination.coefficients.values_mut() {
            *coeff = (*coeff).bit_not();
        }

        self
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
