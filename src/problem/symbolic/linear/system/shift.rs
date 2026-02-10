use super::LinearSystem;

impl LinearSystem {
    pub fn logic_shl(self, rhs: Self) -> Result<Self, ()> {
        // we can only shift when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.logic_shl(b))
    }

    pub fn logic_shr(self, rhs: Self) -> Result<Self, ()> {
        // we can only shift when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.logic_shr(b))
    }

    pub fn arith_shr(self, rhs: Self) -> Result<Self, ()> {
        // we can only shift when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.arith_shr(b))
    }
}
