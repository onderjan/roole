use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::TypedCmp},
    problem::domain::LinearBitvector,
};

impl TypedCmp for LinearBitvector {
    type Output = LinearBitvector;

    fn ult(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(bound)
    }

    fn ule(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(bound)
    }

    fn slt(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(bound)
    }

    fn sle(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: comparison
        Self::Top(bound)
    }
}
