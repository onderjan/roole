use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain},
        traits::forward::{Bitwise, TypedEq},
    },
    problem::{domain::OperationDomain, operation::LinearSystem},
};

impl TypedEq for OperationDomain {
    type Output = OperationDomain;
    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound(), rhs.bound());

        let (Ok(lhs), Ok(rhs)) = (self.try_combination(), rhs.try_combination()) else {
            return OperationDomain::top(RBound::single_bit_bound());
        };

        OperationDomain::from_system(LinearSystem::from_eq(lhs, rhs))
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }

    fn ite(condition: Self::Output, then_branch: Self, else_branch: Self) -> Self {
        assert_eq!(condition.bound().width(), 1);
        let bound = then_branch.bound();
        assert_eq!(bound, else_branch.bound());

        // TODO: simplify if-then-else
        OperationDomain::Top(bound)
    }
}
