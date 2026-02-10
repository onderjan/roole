use super::{
    SymbolicDomain,
    linear::{LinearPolynomial, LinearSystem},
};
use crate::domain::{
    bitvector::{abstr::BitvectorDomain, concr::ConcreteBitvector},
    traits::forward::{HwArith, TypedCmp},
};

impl TypedCmp for SymbolicDomain {
    type Output = SymbolicDomain;

    fn ult(self, rhs: Self) -> Self::Output {
        self.binary_op_try(rhs, |lhs, rhs| lhs.ult(rhs))
    }

    fn ule(self, rhs: Self) -> Self::Output {
        self.binary_op_try(rhs, |lhs, rhs| lhs.ule(rhs))
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

fn signed_cmp_by_unsigned(
    lhs: SymbolicDomain,
    rhs: SymbolicDomain,
    unsigned_func: fn(LinearSystem, LinearSystem) -> Result<LinearSystem, ()>,
) -> SymbolicDomain {
    let bound = lhs.bound();
    assert_eq!(bound, rhs.bound());

    // to convert to unsigned comparison, add overhalf to both
    let overhalf = ConcreteBitvector::new_overhalf(bound);
    let overhalf = SymbolicDomain::from_polynomial(LinearPolynomial::from_constant(overhalf));

    let lhs = lhs.add(overhalf.clone());
    let rhs = rhs.add(overhalf);

    // use the corresponding unsigned comparison
    lhs.binary_op_try(rhs, unsigned_func)
}
