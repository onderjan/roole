use crate::{
    domain::{bitvector::abstr::BitvectorDomain, traits::forward::HwArith},
    problem::{domain::OperationDomain, operation::LinearCombination},
};

impl HwArith for OperationDomain {
    fn arith_neg(self) -> Self {
        let combination = match self.try_combination() {
            Ok(ok) => ok,
            Err(err) => return err,
        };

        Self::from_combination(combination.arith_neg())
    }
    fn add(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.add(b))
    }

    fn sub(self, rhs: Self) -> Self {
        self.linear_combine(rhs, |a, b| a.sub(b))
    }

    fn mul(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (Ok(lhs), Ok(rhs)) = (self.try_combination(), rhs.try_combination()) else {
            // return top value
            return Self::top(bound);
        };

        let (constant, mut combination) = if let Some(constant) = lhs.constant_value() {
            (constant, rhs)
        } else if let Some(constant) = rhs.constant_value() {
            (constant, lhs)
        } else {
            // return top value
            return Self::top(bound);
        };

        // multiply combination by constant
        combination.scale(constant);
        Self::from_combination(combination)
    }

    fn udiv(self, _rhs: Self) -> Self {
        todo!("udiv")
    }

    fn sdiv(self, _rhs: Self) -> Self {
        todo!("sdiv")
    }

    fn urem(self, _rhs: Self) -> Self {
        todo!("urem")
    }

    fn srem(self, _rhs: Self) -> Self {
        todo!("srem")
    }
}

impl OperationDomain {
    fn linear_combine(
        self,
        rhs: OperationDomain,
        op: fn(LinearCombination, LinearCombination) -> LinearCombination,
    ) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (Ok(lhs), Ok(rhs)) = (self.try_combination(), rhs.try_combination()) else {
            return OperationDomain::top(bound);
        };

        let combination = op(lhs, rhs);
        OperationDomain::from_combination(combination)
    }
}
