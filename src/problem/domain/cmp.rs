use super::linear::{LinearExpression, LinearPolynomial};
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::forward::{HwArith, TypedCmp},
    },
    problem::domain::OperationDomain,
};

impl TypedCmp for OperationDomain {
    type Output = OperationDomain;

    fn ult(self, rhs: Self) -> Self::Output {
        unsigned_cmp(self, rhs, |lhs, rhs| lhs.ult(rhs))
    }

    fn ule(self, rhs: Self) -> Self::Output {
        unsigned_cmp(self, rhs, |lhs, rhs| lhs.ule(rhs))
    }

    fn slt(self, rhs: Self) -> Self::Output {
        // convert to unsigned less-than
        signed_cmp_by_unsigned(self, rhs, |lhs, rhs| lhs.ult(rhs))
    }

    fn sle(self, rhs: Self) -> Self::Output {
        // convert to unsigned less-or-equal
        signed_cmp_by_unsigned(self, rhs, |lhs, rhs| lhs.ule(rhs))
    }
}

fn unsigned_cmp(
    lhs: OperationDomain,
    rhs: OperationDomain,
    func: fn(LinearExpression, LinearExpression) -> Result<LinearExpression, ()>,
) -> OperationDomain {
    let bound = lhs.bound();
    assert_eq!(bound, rhs.bound());

    let (Ok(lhs), Ok(rhs)) = (lhs.try_into_expression(), rhs.try_into_expression()) else {
        return OperationDomain::Top(RBound::single_bit_bound());
    };

    if let Ok(result) = (func)(lhs, rhs) {
        OperationDomain::from_expression(result)
    } else {
        OperationDomain::Top(RBound::single_bit_bound())
    }
}

fn signed_cmp_by_unsigned(
    lhs: OperationDomain,
    rhs: OperationDomain,
    unsigned_func: fn(LinearExpression, LinearExpression) -> Result<LinearExpression, ()>,
) -> OperationDomain {
    let bound = lhs.bound();
    assert_eq!(bound, rhs.bound());

    // to convert to signed comparison, add overhalf to both
    let overhalf = ConcreteBitvector::new_overhalf(bound);
    let overhalf = OperationDomain::from_polynomial(LinearPolynomial::from_constant(overhalf));

    let lhs = lhs.add(overhalf.clone());
    let rhs = rhs.add(overhalf);

    unsigned_cmp(lhs, rhs, unsigned_func)
}
