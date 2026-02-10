use crate::domain::bitvector::BitvectorBound;

use super::LinearSystem;

impl LinearSystem {
    pub fn arith_neg(self) -> Result<Self, ()> {
        if self.bound.width() <= 1 {
            // if the width is 0, we can safely return self
            // if the width is 1, we also can since arithmetic negation
            // is considered in two's complement, i.e. !x + 1
            // (!0 + 1) mod 2 = 0, (!1 + 1) mod 2 = 1
            // thus, no negation is done
            return Ok(self);
        }

        if let Ok(expression) = self.try_into_expression() {
            // just process the expression
            Ok(LinearSystem::from_expression(expression.arith_neg()))
        } else {
            // negating a system connected with bitwise operations
            Err(())
        }
    }

    pub fn add(self, rhs: Self) -> Result<Self, ()> {
        // we can only add when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.add(b))
    }

    pub fn sub(self, rhs: Self) -> Result<Self, ()> {
        // we can only subtract when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.sub(b))
    }

    pub fn mul(self, rhs: Self) -> Result<Self, ()> {
        // we can only multiply when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.mul(b))
    }
}
