use super::{LinearExpression, LinearSystem};

impl LinearSystem {
    pub(super) fn expression_binary_op_try<E>(
        self,
        rhs: Self,
        linear_fn: impl Fn(LinearExpression, LinearExpression) -> Result<LinearExpression, E>,
    ) -> Result<Self, ()> {
        let (Ok(lhs), Ok(rhs)) = (self.try_into_expression(), rhs.try_into_expression()) else {
            // at least one of them is not an expression
            return Err(());
        };
        (linear_fn)(lhs, rhs)
            .map(Self::from_expression)
            .map_err(|_| ())
    }
}
