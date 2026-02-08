use crate::{domain::traits::forward::BExt, problem::operation::LinearPolynomial};

impl LinearPolynomial {
    pub fn ite(
        condition: LinearPolynomial,
        then_branch: LinearPolynomial,
        else_branch: LinearPolynomial,
    ) -> Result<LinearPolynomial, ()> {
        let bound = then_branch.bound();
        assert_eq!(bound, else_branch.bound());

        // we can represent ite as condition * (then - else) + else
        // set truth = then - else

        let mut truth = then_branch.sub(else_branch.clone());

        // if condition is constant, we can simplify ite to a polynomial
        if let Some(condition) = condition.constant_value() {
            // condition is constant, scale truth by it (zero-extended) and add else branch
            truth.scale(condition.uext(bound));

            return Ok(truth.add(else_branch));
        }

        // if truth is constant and we can unsigned-extend condition to truth size,
        // we can simplify ite to a polynomial
        if let Some(truth) = truth.constant_value()
            && let Ok(mut extended_conditional) = condition.uext(bound)
        {
            // truth is constant, scale condition by it and add else branch
            extended_conditional.scale(truth);

            return Ok(extended_conditional.add(else_branch));
        }

        Err(())
    }
}
