use super::{SymbolicDomain, linear::LinearSystem};
use crate::domain::{
    bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
    traits::forward::{Bitwise, TypedEq},
};

impl TypedEq for SymbolicDomain {
    type Output = SymbolicDomain;
    fn eq(self, rhs: Self) -> Self::Output {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        // we can simplify if we are working with Booleans and at least one is a constant
        if bound.width() == 1 {
            let (lhs_value, rhs_value) = (self.constant_value(), rhs.constant_value());

            match (lhs_value, rhs_value) {
                (None, None) => {}
                (None, Some(rhs_value)) => return equality_result(rhs_value, self),
                (Some(lhs_value), None) => return equality_result(lhs_value, rhs),
                (Some(lhs_value), Some(rhs_value)) => {
                    // just combine
                    return Self::from_bool(lhs_value == rhs_value);
                }
            };
        }

        // otherwise, resolve in linear system
        self.binary_op_try(rhs, |a, b| a.typed_eq(b))
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
            SymbolicDomain::Linear(condition),
            SymbolicDomain::Linear(then_branch),
            SymbolicDomain::Linear(else_branch),
        ) = (condition, then_branch, else_branch)
        else {
            return SymbolicDomain::Top(bound);
        };

        if let Ok(result) = LinearSystem::ite(condition, then_branch, else_branch) {
            SymbolicDomain::Linear(result)
        } else {
            SymbolicDomain::Top(bound)
        }
    }
}

fn equality_result(constant: ConcreteBitvector<RBound>, other: SymbolicDomain) -> SymbolicDomain {
    if constant.is_nonzero() {
        // equality of form 1 == other
        // just return the other
        other
    } else {
        // equality of form 0 == other
        // bit-not other
        other.bit_not()
    }
}

fn simplify_ite_boolean_branches(
    condition: SymbolicDomain,
    then_branch: bool,
    else_branch: bool,
) -> SymbolicDomain {
    if then_branch == else_branch {
        // tautology (both true) or contradiction (both false)
        SymbolicDomain::from_bool(then_branch)
    } else if then_branch {
        // identity (take then if true, take else if false)
        condition
    } else {
        // bitwise not (take then if false, take else if true)
        condition.bit_not()
    }
}
