use super::{super::LinearRelation, LinearExpression};
impl LinearExpression {
    pub fn typed_eq(self, rhs: Self) -> Result<Self, ()> {
        // the constant simplification is already handled by the symbolic domain
        // otherwise, we can only do equality between polynomials

        if let (LinearExpression::Polynomial(lhs), LinearExpression::Polynomial(rhs)) = (self, rhs)
        {
            Ok(LinearExpression::Relation(LinearRelation::from_eq(
                lhs, rhs,
            )))
        } else {
            Err(())
        }
    }
}
