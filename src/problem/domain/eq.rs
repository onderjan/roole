use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain},
        traits::forward::{BExt, Bitwise, TypedEq},
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

        let (lhs, rhs) = match (self.try_polynomial(), rhs.try_polynomial()) {
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

        if bound.width() == 0 {
            // replace with empty
            return OperationDomain::from_polynomial(LinearPolynomial::empty(bound));
        }

        let condition = match condition.try_polynomial() {
            Ok(condition) => {
                if let Some(condition_value) = condition.constant_value() {
                    // constant condition value, select the branch
                    if condition_value.is_nonzero() {
                        return then_branch;
                    } else {
                        return else_branch;
                    }
                }
                // go back
                OperationDomain::from_polynomial(condition)
            }
            Err(condition) => condition,
        };

        // try to simplify with polynomial branches
        let (Ok(then_branch), Ok(else_branch)) =
            (then_branch.try_polynomial(), else_branch.try_polynomial())
        else {
            return OperationDomain::Top(bound);
        };

        // collapse to condition if the width is a single bit
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

        let Ok(condition) = condition.try_polynomial() else {
            return OperationDomain::Top(bound);
        };

        // we can represent ite as condition * (then - else) + else
        // set truth = then - else
        // then, if either condition or truth is a constant,
        // we can simplify ite to a polynomial

        let mut truth = then_branch.sub(else_branch.clone());

        if let Some(truth) = truth.constant_value() {
            let Ok(mut conditional_truth) = condition.uext(bound) else {
                // could not extend the condition
                return OperationDomain::Top(bound);
            };

            // truth is constant, scale condition by it and add else branch
            conditional_truth.scale(truth);

            let result = OperationDomain::from_polynomial(conditional_truth.add(else_branch));

            return result;
        };

        if let Some(condition) = condition.constant_value() {
            // condition is constant, scale truth by it (zero-extended) and add else branch
            truth.scale(condition.uext(bound));

            let result = OperationDomain::from_polynomial(truth.add(else_branch));

            return result;
        }

        OperationDomain::Top(bound)
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
