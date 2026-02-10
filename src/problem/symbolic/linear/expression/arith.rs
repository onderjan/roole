use super::LinearExpression;

impl LinearExpression {
    pub fn arith_neg(self) -> Self {
        match self {
            LinearExpression::Polynomial(polynomial) => {
                // arithmetically negate the polynomial
                LinearExpression::Polynomial(polynomial.arith_neg())
            }
            LinearExpression::Relation(_) => {
                // arithmetic negation is considered in two's , i.e. !x + 1
                // relations have a single-bit result
                //(!0 + 1) mod 2 = 0, (!1 + 1) mod 2 = 1
                // thus, no negation is done
                self
            }
        }
    }

    pub fn add(self, rhs: Self) -> Result<Self, ()> {
        // can only add polynomials, not relations
        self.polynomial_binary_op(rhs, |a, b| a.add(b))
    }

    pub fn sub(self, rhs: Self) -> Result<Self, ()> {
        // can only subtract polynomials, not relations
        self.polynomial_binary_op(rhs, |a, b| a.sub(b))
    }

    pub fn mul(self, rhs: Self) -> Result<Self, ()> {
        // can only multiply polynomials, not relations
        self.polynomial_binary_op_try(rhs, |a, b| a.mul(b))
    }
}
