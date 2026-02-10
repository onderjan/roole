use super::LinearSystem;

impl LinearSystem {
    pub fn typed_eq(self, rhs: Self) -> Result<Self, ()> {
        // we can only compute typed equality when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.typed_eq(b))
    }
}
