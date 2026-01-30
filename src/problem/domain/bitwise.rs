use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::Bitwise},
    problem::{domain::OperationDomain, operation::LinearSystem},
};

impl Bitwise for OperationDomain {
    fn bit_not(self) -> Self {
        let linear = match self {
            OperationDomain::Top(_) => return self,
            OperationDomain::Linear(linear) => linear,
        };

        OperationDomain::Linear(linear.bit_not())
    }

    fn bit_and(self, rhs: Self) -> Self {
        self.bit_linear(rhs, |a, b| a.and(b))
    }
    fn bit_or(self, rhs: Self) -> Self {
        self.bit_linear(rhs, |a, b| a.or(b))
    }
    fn bit_xor(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: handle masking situations

        OperationDomain::top(bound)
    }
}

impl OperationDomain {
    fn bit_linear(
        self,
        rhs: Self,
        op: fn(LinearSystem, LinearSystem) -> Option<LinearSystem>,
    ) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (Ok(lhs), Ok(rhs)) = (self.try_system(), rhs.try_system()) else {
            return Self::top(bound);
        };

        let Some(system) = op(lhs, rhs) else {
            return Self::top(bound);
        };

        OperationDomain::from_system(system)
    }
}
