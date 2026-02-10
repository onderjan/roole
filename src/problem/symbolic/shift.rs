use super::SymbolicDomain;
use crate::domain::traits::forward::HwShift;

impl HwShift for SymbolicDomain {
    type Output = Self;

    fn logic_shl(self, amount: Self) -> Self {
        self.binary_op_try(amount, |lhs, amount| lhs.logic_shl(amount), false)
    }

    fn logic_shr(self, amount: Self) -> Self {
        self.binary_op_try(amount, |lhs, amount| lhs.logic_shr(amount), false)
    }

    fn arith_shr(self, amount: Self) -> Self {
        self.binary_op_try(amount, |lhs, amount| lhs.arith_shr(amount), false)
    }
}
