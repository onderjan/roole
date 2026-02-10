use super::LinearExpression;

impl LinearExpression {
    pub fn logic_shl(self, rhs: Self) -> Result<Self, ()> {
        // can only shift polynomials, not relations
        self.polynomial_binary_op_try(rhs, |a, b| a.logic_shl(b))
    }

    pub fn logic_shr(self, rhs: Self) -> Result<Self, ()> {
        // can only shift polynomials, not relations
        self.polynomial_binary_op_try(rhs, |a, b| a.logic_shr(b))
    }

    pub fn arith_shr(self, rhs: Self) -> Result<Self, ()> {
        // can only shift polynomials, not relations
        self.polynomial_binary_op_try(rhs, |a, b| a.arith_shr(b))
    }
}
