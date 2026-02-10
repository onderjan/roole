use crate::problem::symbolic::linear::LinearPolynomial;

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

    pub fn ite(condition: Self, then_branch: Self, else_branch: Self) -> Result<Self, ()> {
        // try to convert to a system if all are polynomials
        let (
            LinearExpression::Polynomial(condition),
            LinearExpression::Polynomial(then_branch),
            LinearExpression::Polynomial(else_branch),
        ) = (condition, then_branch, else_branch)
        else {
            return Err(());
        };

        LinearPolynomial::ite(condition, then_branch, else_branch).map(LinearExpression::Polynomial)
    }
}
