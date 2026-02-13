use crate::{domain::traits::forward::HwArith, problem::symbolic::SymbolicDomain};

impl HwArith for SymbolicDomain {
    fn arith_neg(self) -> Self {
        self.unary_op_try(|system| system.arith_neg())
    }

    fn add(self, rhs: Self) -> Self {
        self.binary_op_try(rhs, |a, b| a.add(b), false)
    }

    fn sub(self, rhs: Self) -> Self {
        self.binary_op_try(rhs, |a, b| a.sub(b), false)
    }

    fn mul(self, rhs: Self) -> Self {
        self.binary_op_try(rhs, |a, b| a.mul(b), false)
    }

    fn udiv_wrapping_or_full(self, _rhs: Self) -> Self {
        todo!("udiv")
    }

    fn sdiv_wrapping_or_full(self, _rhs: Self) -> Self {
        todo!("sdiv")
    }

    fn urem_wrapping_or_dividend(self, _rhs: Self) -> Self {
        todo!("urem")
    }

    fn srem_wrapping_or_dividend(self, _rhs: Self) -> Self {
        todo!("srem")
    }
}
