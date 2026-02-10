use super::LinearPolynomial;

impl LinearPolynomial {
    pub fn into_normal_form(mut self) -> Self {
        // eliminate zero coefficients
        self.linear_terms.retain(|_, coeff| !coeff.is_zero());
        self
    }

    pub fn might_overflow(&self) -> bool {
        if self.linear_terms.is_empty() {
            // only constant, definitely cannot overflow
            return false;
        }

        // TODO: determine if the polynomial might overflow more finely

        let Some((monomial, constant)) = self.monomial_and_constant_value() else {
            // we are unsure, return true
            return true;
        };

        let Some(monomial) = monomial else {
            // just a constant, definitely cannot overflow
            return false;
        };

        if constant.is_nonzero() {
            // we are unsure, return true
            return true;
        }

        monomial.might_overflow()
    }
}
