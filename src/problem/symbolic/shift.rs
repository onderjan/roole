use super::{SymbolicDomain, linear::LinearPolynomial};
use crate::domain::{bitvector::abstr::BitvectorDomain, traits::forward::HwShift};

impl HwShift for SymbolicDomain {
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
    lhs: SymbolicDomain,
    amount: SymbolicDomain,
    polynomial_func: fn(LinearPolynomial, LinearPolynomial) -> Result<LinearPolynomial, ()>,
) -> SymbolicDomain {
    let bound = amount.bound();
    assert_eq!(bound, amount.bound());

    let (Ok(lhs), Ok(amount)) = (lhs.try_into_polynomial(), amount.try_into_polynomial()) else {
        return SymbolicDomain::top(bound);
    };

    match (polynomial_func)(lhs, amount) {
        Ok(result) => SymbolicDomain::from_polynomial(result),
        Err(()) => SymbolicDomain::top(bound),
    }
}
