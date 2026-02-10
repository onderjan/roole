use super::LinearSystem;

impl LinearSystem {
    pub fn ult(self, rhs: Self) -> Result<Self, ()> {
        // we can only compute unsigned-less-than when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.ult(b))
    }

    pub fn ule(self, rhs: Self) -> Result<Self, ()> {
        // we can only compute unsigned-less-or-equal when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.ule(b))
    }
}
