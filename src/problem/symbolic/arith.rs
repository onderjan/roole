use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::HwArith},
    problem::symbolic::SymbolicDomain,
};

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

    fn udiv_wrapping_or_all_ones(self, rhs: Self) -> Self {
        // TODO: division in symbolic domain
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        Self::Top(bound)
    }

    fn sdiv_wrapping_by_quadrants(self, rhs: Self) -> Self {
        // TODO: division in symbolic domain
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        Self::Top(bound)
    }

    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self {
        // TODO: division in symbolic domain
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        Self::Top(bound)
    }

    fn srem_wrapping_by_quadrants(self, rhs: Self) -> Self {
        // TODO: division in symbolic domain
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        Self::Top(bound)
    }
}
