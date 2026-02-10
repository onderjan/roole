use super::{super::LinearPolynomial, LinearExpression};

impl LinearExpression {
    pub(super) fn polynomial_binary_op(
        self,
        rhs: Self,
        linear_fn: impl Fn(LinearPolynomial, LinearPolynomial) -> LinearPolynomial,
    ) -> Result<Self, ()> {
        if let (LinearExpression::Polynomial(lhs), LinearExpression::Polynomial(rhs)) = (self, rhs)
        {
            Ok(Self::Polynomial((linear_fn)(lhs, rhs)))
        } else {
            // at least one of them is not a polynomial
            Err(())
        }
    }

    pub(super) fn polynomial_binary_op_try<E>(
        self,
        rhs: Self,
        linear_fn: impl Fn(LinearPolynomial, LinearPolynomial) -> Result<LinearPolynomial, E>,
    ) -> Result<Self, ()> {
        if let (LinearExpression::Polynomial(lhs), LinearExpression::Polynomial(rhs)) = (self, rhs)
        {
            (linear_fn)(lhs, rhs).map(Self::Polynomial).map_err(|_| ())
        } else {
            // at least one of them is not a polynomial
            Err(())
        }
    }
}
