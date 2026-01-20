use crate::domain::{
    bitvector::{BitvectorBound, abstr::linear::LinearBitvector},
    traits::Join,
};

impl<B: BitvectorBound> Join for LinearBitvector<B> {
    fn join(self, other: &Self) -> Self {
        todo!()
    }
}
