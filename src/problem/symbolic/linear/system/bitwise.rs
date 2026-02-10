use super::{super::LinearExpression, LinearSystem};

impl LinearSystem {
    pub fn bit_not(mut self) -> Self {
        // bit-negate the junction type
        self.conjunction = !self.conjunction;

        // bit-negate each expression
        for expression in std::mem::take(&mut self.expressions) {
            self.expressions.push(expression.bit_not());
        }

        self
    }

    pub fn bit_junction(self, rhs: Self, conjunction: bool) -> Result<Self, ()> {
        let bound = self.bound;
        assert_eq!(bound, rhs.bound);

        let lhs_single = self.expressions.len() == 1;
        let rhs_single = rhs.expressions.len() == 1;

        // can only combine if both sides are compatible with the junction type (conjunction/disjunction)
        // a side is compatible if it has the same junction type or has exactly one expression
        let lhs_compatible = self.conjunction == conjunction || lhs_single;
        let rhs_compatible = rhs.conjunction == conjunction || rhs_single;

        if !lhs_compatible || !rhs_compatible {
            return Err(());
        }

        if lhs_single
            && rhs_single
            && let (LinearExpression::Polynomial(lhs), LinearExpression::Polynomial(rhs)) =
                (&self.expressions[0], &rhs.expressions[0])
        {
            // we can try to combine the polynomials
            if let Ok(result) = lhs.clone().bitwise_combine(rhs.clone(), conjunction) {
                return Ok(Self::from_polynomial(result));
            }
        }

        // combine the expressions
        let mut expressions = self.expressions;
        expressions.extend(rhs.expressions);

        Ok(Self {
            conjunction,
            bound,
            expressions,
        })
    }
}
