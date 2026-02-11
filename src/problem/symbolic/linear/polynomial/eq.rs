use super::LinearPolynomial;
use crate::domain::{bitvector::BitvectorBound, traits::forward::BExt};

impl LinearPolynomial {
    pub fn ite(
        condition: LinearPolynomial,
        mut then_branch: LinearPolynomial,
        mut else_branch: LinearPolynomial,
    ) -> Result<LinearPolynomial, ()> {
        let bound = then_branch.bound();
        assert_eq!(bound, else_branch.bound());
        assert_eq!(condition.bound().width(), 1);

        let not_condition = condition.clone().bit_not();

        // since we assume polynomials equal to zero, we will assume that
        // the bit-not condition is zero on then branch taken
        // and that condition is zero on else branch taken

        then_branch.assume_polynomial_is_zero(&not_condition);
        else_branch.assume_polynomial_is_zero(&condition);

        // we can represent ite as else + condition * (then - else)
        // set truth = (then - else)

        let mut truth = then_branch.clone().sub(else_branch.clone());

        // if condition is constant, we can simplify ite to a polynomial
        if let Some(condition) = condition.constant_value() {
            // condition is constant, scale truth by it (zero-extended) and add else branch
            truth.scale(condition.uext(bound));

            return Ok(truth.add(else_branch));
        }

        let Some(truth) = truth.constant_value() else {
            // truth is not constant, cannot do much
            return Err(());
        };

        // if we can unsigned-extend condition to truth size and we can get truth constant value with it assumed
        // we can simplify ite to a polynomial
        if let Ok(mut extended_condition) = condition.clone().uext(bound) {
            // simplify as else + condition * truth
            extended_condition.scale(truth);
            return Ok(else_branch.add(extended_condition));
        }

        // try the same thing with negated condition

        if let Ok(mut extended_not_condition) = not_condition.uext(bound) {
            // simplify as then + not_condition * (else - then)
            // since truth is (then - else)
            // represent as then - not_condition * truth
            extended_not_condition.scale(truth);

            return Ok(then_branch.sub(extended_not_condition));
        }

        Err(())
    }
}
