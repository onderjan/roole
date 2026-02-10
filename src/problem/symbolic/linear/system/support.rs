use std::fmt::{Debug, UpperHex};

use itertools::Itertools;

use super::{LinearExpression, LinearPolynomial, LinearSystem};

impl LinearSystem {
    pub(super) fn expression_binary_op_try<E>(
        self,
        rhs: Self,
        linear_fn: impl Fn(LinearExpression, LinearExpression) -> Result<LinearExpression, E>,
    ) -> Result<Self, ()> {
        let (Ok(lhs), Ok(rhs)) = (self.try_into_expression(), rhs.try_into_expression()) else {
            // at least one of them is not an expression
            return Err(());
        };
        (linear_fn)(lhs, rhs)
            .map(Self::from_expression)
            .map_err(|_| ())
    }

    pub(super) fn from_expression(expression: LinearExpression) -> Self {
        let expression = expression.into_normal_form();
        Self {
            conjunction: true,
            bound: expression.bound(),
            expressions: vec![expression],
        }
    }

    pub(super) fn from_polynomial(polynomial: LinearPolynomial) -> Self {
        Self::from_expression(LinearExpression::Polynomial(polynomial))
    }

    pub(super) fn try_into_expression(self) -> Result<LinearExpression, LinearSystem> {
        if self.expressions.len() != 1
            || !matches!(self.expressions[0], LinearExpression::Polynomial(_))
        {
            return Err(self);
        }

        let Ok(expression) = self.expressions.into_iter().exactly_one() else {
            panic!("Should be ensured to be a polynomial");
        };
        Ok(expression)
    }

    pub(crate) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        let op_symbol = if self.conjunction { "&" } else { "|" };

        if self.expressions.is_empty() {
            return write!(f, "({})", op_symbol);
        }

        if self.expressions.len() == 1 {
            return self.expressions[0].format(f, hex);
        }

        let mut is_first = true;
        for expression in &self.expressions {
            if is_first {
                is_first = false;
            } else {
                write!(f, " {} ", op_symbol)?;
            }

            match expression {
                LinearExpression::Polynomial(polynomial) => polynomial.format(f, hex)?,
                LinearExpression::Relation(relation) => {
                    // surround with parentheses
                    write!(f, "(")?;
                    relation.format(f, hex)?;
                    write!(f, ")")?;
                }
            }
        }
        Ok(())
    }
}

impl Debug for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
