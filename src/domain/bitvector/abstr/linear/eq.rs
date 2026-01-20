use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{BitvectorDomain, linear::LinearBitvector},
    },
    traits::forward::{Bitwise, TypedEq},
};

impl<B: BitvectorBound> TypedEq for LinearBitvector<B> {
    type Output = LinearBitvector<B::SingleBit>;
    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);

        // this is not really a linear combination for arbitrary widths
        // TODO: allow resolving equality somewhat
        Self::Output::top(B::single_bit_bound())
    }

    fn ne(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
