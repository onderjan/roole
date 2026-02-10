use super::{super::LinearExpression, LinearSystem};

impl LinearSystem {
    pub fn typed_eq(self, rhs: Self) -> Result<Self, ()> {
        // we can only compute typed equality when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.typed_eq(b))
    }

    pub fn ite(condition: Self, then_branch: Self, else_branch: Self) -> Result<Self, ()> {
        // try to convert to a system if all are expressions
        let (Ok(condition), Ok(then_branch), Ok(else_branch)) = (
            condition.try_into_expression(),
            then_branch.try_into_expression(),
            else_branch.try_into_expression(),
        ) else {
            return Err(());
        };

        LinearExpression::ite(condition, then_branch, else_branch)
            .map(LinearSystem::from_expression)
    }
}
