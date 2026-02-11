use super::{super::LinearExpression, LinearSystem};

impl LinearSystem {
    pub fn typed_eq(self, rhs: Self) -> Result<Self, ()> {
        // we can only compute typed equality when the system consists of a single expression
        self.expression_binary_op_try(rhs, |a, b| a.typed_eq(b))
    }

    pub fn ite(condition: Self, then_branch: Self, else_branch: Self) -> Result<Self, ()> {
        // try to figure out if the result of then and else branch is the same constant if the condition is true
        // if it is, we can replace the then branch with else
        // and in turn, the whole condition with the else branch

        if can_simplify_taker(&condition, &then_branch, &else_branch) {
            return Ok(else_branch);
        }

        // do the same thing but if the condition is false
        // if then and else branch give the same results, we can replace the else branch with then
        // and in turn, the whole condition with the then branch

        let not_condition = condition.clone().bit_not();
        if can_simplify_taker(&not_condition, &else_branch, &then_branch) {
            return Ok(then_branch);
        }

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

fn can_simplify_taker(
    taker: &LinearSystem,
    taken_branch: &LinearSystem,
    not_taken_branch: &LinearSystem,
) -> bool {
    let mut then_when_true = taken_branch.clone();
    let mut else_when_true = not_taken_branch.clone();

    then_when_true.assume(taker);
    else_when_true.assume(taker);

    then_when_true == else_when_true
}
