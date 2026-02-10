use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain},
        traits::forward::{Bitwise, TypedEq},
    },
    problem::{
        domain::OperationDomain,
        operation::{LinearPolynomial, LinearRelation, LinearSystem},
    },
};

impl TypedEq for OperationDomain {
    type Output = OperationDomain;
    fn eq(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        let (lhs, rhs) = match (self.try_into_polynomial(), rhs.try_into_polynomial()) {
            (Err(_), Err(_)) => {
                // cannot combine
                return OperationDomain::top(RBound::single_bit_bound());
            }
            (Ok(polynomial), Err(other)) | (Err(other), Ok(polynomial)) => {
                // we can simplify if we are working with Booleans and polynomial is a constant
                if bound.width() == 1
                    && let Some(constant) = polynomial.constant_value()
                {
                    if constant.is_nonzero() {
                        // equality of form 1 == other
                        // just return the other
                        return other;
                    } else {
                        // equality of form 0 == other
                        // bit-not other
                        return other.bit_not();
                    }
                };

                // cannot combine
                return OperationDomain::top(RBound::single_bit_bound());
            }
            (Ok(lhs), Ok(rhs)) => (lhs, rhs),
        };

        OperationDomain::Linear(LinearSystem::from_relation(LinearRelation::from_eq(
            lhs, rhs,
        )))
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }

    fn ite(condition: Self::Output, then_branch: Self, else_branch: Self) -> Self {
        assert_eq!(condition.bound().width(), 1);
        let bound = then_branch.bound();
        assert_eq!(bound, else_branch.bound());

        // select the branch if condition value is constant
        if let Some(condition_value) = condition.constant_value() {
            if condition_value.is_nonzero() {
                return then_branch;
            } else {
                return else_branch;
            }
        }

        // collapse to condition if both then and else branches are constant Booleans
        if bound.width() == 1
            && let (Some(then_branch), Some(else_branch)) =
                (then_branch.constant_value(), else_branch.constant_value())
        {
            return simplify_ite_boolean_branches(
                condition,
                then_branch.is_nonzero(),
                else_branch.is_nonzero(),
            );
        }

        // try to forward to LinearSystem
        let (
            OperationDomain::Linear(condition),
            OperationDomain::Linear(then_branch),
            OperationDomain::Linear(else_branch),
        ) = (condition, then_branch, else_branch)
        else {
            return OperationDomain::Top(bound);
        };

        if let Ok(result) = LinearSystem::ite(condition, then_branch, else_branch) {
            OperationDomain::Linear(result)
        } else {
            OperationDomain::Top(bound)
        }
    }
}

fn simplify_ite_boolean_branches(
    condition: OperationDomain,
    then_branch: bool,
    else_branch: bool,
) -> OperationDomain {
    if then_branch == else_branch {
        // tautology (both true) or contradiction (both false)
        OperationDomain::from_polynomial(LinearPolynomial::single_bit(then_branch))
    } else if then_branch {
        // identity (take then if true, take else if false)
        condition
    } else {
        // bitwise not (take then if false, take else if true)
        condition.bit_not()
    }
}
