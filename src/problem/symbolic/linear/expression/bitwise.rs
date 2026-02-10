use super::super::LinearExpression;

impl LinearExpression {
    pub fn bit_not(self) -> Self {
        match self {
            LinearExpression::Polynomial(polynomial) => {
                LinearExpression::Polynomial(polynomial.bit_not())
            }
            LinearExpression::Relation(relation) => match relation.bit_not() {
                Ok(relation) => LinearExpression::Relation(relation),
                Err(polynomial) => LinearExpression::Polynomial(polynomial),
            },
        }
    }
}
