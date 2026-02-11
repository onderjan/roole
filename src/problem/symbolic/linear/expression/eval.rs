use std::collections::BTreeMap;

use super::LinearExpression;
use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

impl LinearExpression {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.evaluate(fetch),
            LinearExpression::Relation(relation) => relation.evaluate(fetch),
        }
    }

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.constant_value(),
            LinearExpression::Relation(_) => None,
        }
    }

    pub fn assume(&mut self, assumptions: &[Self]) {
        let mut concrete_assumptions = BTreeMap::new();

        for assumption in assumptions {
            match assumption {
                LinearExpression::Polynomial(_) => {
                    // TODO: do polynomial assumptions
                }
                LinearExpression::Relation(linear_relation) => {
                    if linear_relation.slack().is_zero() {
                        // equality
                        if let Some((Some(monomial), constant)) = linear_relation
                            .clone()
                            .into_polynomial()
                            .monomial_and_constant_value()
                        {
                            // we have monomial + constant = 0 (mod bound)
                            // therefore, monomial = -constant (mod bound)
                            // as monomial = slice * coefficient, we will just do it with coefficient 0 for now

                            if monomial.coefficient.is_one() {
                                concrete_assumptions.insert(monomial.slice, constant.arith_neg());
                            }
                        }
                    }
                }
            }
        }

        let polynomial = match self {
            LinearExpression::Polynomial(polynomial) => polynomial,
            LinearExpression::Relation(_) => {
                // TODO: assume with relation
                return;
            }
        };

        for (assumed_slice, assumed_value) in concrete_assumptions {
            polynomial.assume_slice_value(assumed_slice, assumed_value);
        }
    }
}
