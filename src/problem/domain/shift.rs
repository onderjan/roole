use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::HwShift},
    problem::{domain::OperationDomain, operation::LinearPolynomial},
};

impl HwShift for OperationDomain {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        perform_shift(self, amount, |lhs, amount| lhs.logic_shl(amount))
    }

    fn logic_shr(self, amount: Self) -> Self {
        perform_shift(self, amount, |lhs, amount| lhs.logic_shr(amount))
    }

    fn arith_shr(self, amount: Self) -> Self {
        perform_shift(self, amount, |lhs, amount| lhs.arith_shr(amount))
    }
}

fn perform_shift(
    lhs: OperationDomain,
    amount: OperationDomain,
    polynomial_func: fn(LinearPolynomial, LinearPolynomial) -> Result<LinearPolynomial, ()>,
) -> OperationDomain {
    let bound = amount.bound();
    assert_eq!(bound, amount.bound());

    let (Ok(lhs), Ok(amount)) = (lhs.try_into_polynomial(), amount.try_into_polynomial()) else {
        return OperationDomain::top(bound);
    };

    match (polynomial_func)(lhs, amount) {
        Ok(result) => OperationDomain::from_polynomial(result),
        Err(()) => OperationDomain::top(bound),
    }
}
