use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain},
        traits::forward::TypedCmp,
    },
    problem::domain::OperationDomain,
};

impl TypedCmp for OperationDomain {
    type Output = OperationDomain;

    fn ult(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(RBound::single_bit_bound())
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(RBound::single_bit_bound())
    }

    fn slt(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(RBound::single_bit_bound())
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(RBound::single_bit_bound())
    }
}
