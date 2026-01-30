use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::HwShift},
    problem::domain::OperationDomain,
};

impl HwShift for OperationDomain {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, amount.bound());
        // TODO: shifts
        Self::Top(bound)
    }

    fn logic_shr(self, amount: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, amount.bound());
        // TODO: shifts
        Self::Top(bound)
    }

    fn arith_shr(self, amount: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, amount.bound());
        // TODO: shifts
        Self::Top(bound)
    }
}
