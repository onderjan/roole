use crate::{
    domain::{bitvector::BitvectorBound, traits::forward::BExt},
    problem::operation::LinearPolynomial,
};

impl LinearPolynomial {
    pub fn ite(
        condition: LinearPolynomial,
        then_branch: LinearPolynomial,
        else_branch: LinearPolynomial,
    ) -> Result<LinearPolynomial, ()> {
        let bound = then_branch.bound();
        assert_eq!(bound, else_branch.bound());
        assert_eq!(condition.bound().width(), 1);

        // we can represent ite as else + condition * (then - else)
        // set truth = then - else

        let mut truth = then_branch.clone().sub(else_branch.clone());

        // if condition is constant, we can simplify ite to a polynomial
        if let Some(condition) = condition.constant_value() {
            // condition is constant, scale truth by it (zero-extended) and add else branch
            truth.scale(condition.uext(bound));

            return Ok(truth.add(else_branch));
        }

        // if we can unsigned-extend condition to truth size and we can get truth constant value with it assumed
        // we can simplify ite to a polynomial
        if let Ok(mut extended_condition) = condition.clone().uext(bound)
            && let Some(truth) = truth.constant_value_with_assumption(&condition)
        {
            // truth is constant, scale condition by it and add else branch
            extended_condition.scale(truth);
            return Ok(else_branch.add(extended_condition));
        }

        // try the same thing with negated condition
        // note that the negated condition must be assumed instead of normal

        let not_condition = condition.clone().bit_not();
        if let Ok(mut extended_not_condition) = not_condition.clone().uext(bound)
            && let Some(truth) = truth.constant_value_with_assumption(&not_condition)
        {
            // we can represent as then + not_condition * (else - then)
            // and so as then - not_condition * (then - else)
            extended_not_condition.scale(truth);

            return Ok(then_branch.sub(extended_not_condition));
        }

        Err(())
    }
}
