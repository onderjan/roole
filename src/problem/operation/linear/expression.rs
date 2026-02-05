use std::{collections::BTreeMap, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::{HwArith, TypedCmp},
    },
    problem::{
        eval::EvaluableDomain,
        formula::FormulaId,
        operation::{
            LinearPolynomial,
            linear::{LinearRelation, monomial::LinearMonomial},
        },
    },
};

/// Either a linear polynomial or a linear relation.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearExpression {
    Polynomial(LinearPolynomial),
    Relation(LinearRelation),
}

impl LinearExpression {
    pub fn bound(&self) -> RBound {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.bound(),
            LinearExpression::Relation(_) => {
                // relation result bound is always 1
                RBound::single_bit_bound()
            }
        }
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.evaluate(fetch),
            LinearExpression::Relation(relation) => relation.evaluate(fetch),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.used_ids(),
            LinearExpression::Relation(relation) => relation.used_ids(),
        }
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        match self {
            LinearExpression::Polynomial(polynomial) => polynomial.remap(old_to_new),
            LinearExpression::Relation(relation) => relation.remap(old_to_new),
        }
    }

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

    pub fn into_normal_form(self) -> Self {
        let relation = match self {
            LinearExpression::Polynomial(polynomial) => {
                return LinearExpression::Polynomial(polynomial.into_normal_form());
            }
            LinearExpression::Relation(relation) => relation,
        };

        if relation.slack().is_full_mask() {
            // polynomial <= max_value, this is a tautology
            return LinearExpression::Polynomial(LinearPolynomial::single_bit(true));
        }

        let bound = relation.polynomial().bound();

        match bound.width() {
            0 => {
                // can convert into empty polynomial
                return LinearExpression::Polynomial(LinearPolynomial::empty(bound));
            }
            1 => {
                // can convert into Boolean
                // since we already resolved the case where slack is max_value
                // and max_value is 1 in this case, the slack must be 0 here

                // the relation is left <= 0, i.e. left == 0
                // we must bit-not to obtain (!left) == (!1)
                // i.e. !left == 1, which can be converted to polynomial !left

                return LinearExpression::Polynomial(relation.into_polynomial().bit_not());
            }
            _ => {}
        }

        // width is above 1

        let slack = *relation.slack();

        let Some((monomial, constant)) = relation.polynomial().monomial_and_constant_value() else {
            // cannot convert
            return LinearExpression::Relation(relation);
        };

        let Some(monomial) = monomial else {
            // the result is whether constant <= slack
            return LinearExpression::Polynomial(LinearPolynomial::from_constant(
                constant.ule(slack),
            ));
        };

        let slice = monomial.slice;
        let coefficient = monomial.coefficient;

        // if the monomial is single-bit, we will be able to simplify
        if slice.width.get() != 1 {
            return LinearExpression::Relation(relation);
        }

        let result_if_zero = constant.ule(slack);
        let result_if_one = coefficient.add(constant).ule(slack);

        if result_if_zero == result_if_one {
            // tautology / contradiction
            return LinearExpression::Polynomial(LinearPolynomial::from_constant(result_if_one));
        }

        // if result_if_zero is 0 and result_if_one is 1, we want to construct single_bit
        // if result_if_zero is 1 and result_if_one is 0, we want to construct (single_bit + 1) mod 2
        let constant = result_if_zero;

        let single_bit_one = ConcreteBitvector::one(RBound::single_bit_bound());
        let monomial = LinearMonomial::new(single_bit_one, slice);

        LinearExpression::Polynomial(LinearPolynomial::from_monomial_and_constant(
            monomial, constant,
        ))
    }
}

impl Debug for LinearExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinearExpression::Polynomial(polynomial) => Debug::fmt(polynomial, f),
            LinearExpression::Relation(relation) => Debug::fmt(relation, f),
        }
    }
}
