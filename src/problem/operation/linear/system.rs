use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

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

mod ops;

/// A system of linear relations.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinearSystem {
    Single(LinearRelation),
    Conjunction(Vec<LinearRelation>),
    Disjunction(Vec<LinearRelation>),
}

impl LinearSystem {
    pub fn from_eq(lhs: LinearPolynomial, rhs: LinearPolynomial) -> Self {
        // if both are polynomials, make into an equality

        let polynomial = lhs.sub(rhs);
        let slack = ConcreteBitvector::zero(polynomial.bound());
        LinearSystem::Single(LinearRelation::new(polynomial, slack))
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            LinearSystem::Single(relation) => relation.evaluate(fetch),
            LinearSystem::Conjunction(relations) => {
                Self::evaluate_relations(fetch, relations, true)
            }
            LinearSystem::Disjunction(relations) => {
                Self::evaluate_relations(fetch, relations, false)
            }
        }
    }

    pub fn evaluate_relations<D: EvaluableDomain>(
        fetch: impl Fn(FormulaId) -> D,
        relations: &[LinearRelation],
        universal: bool,
    ) -> D {
        let bound = RBound::new(1);
        let mut result = if universal {
            // start with 1
            D::single_value(ConcreteBitvector::one(bound))
        } else {
            // start with 0
            D::single_value(ConcreteBitvector::zero(bound))
        };

        for relation in relations {
            let relation_result = relation.evaluate(&fetch);
            if universal {
                result = result.bit_and(relation_result);
            } else {
                result = result.bit_or(relation_result);
            }
        }
        result
    }

    pub fn normalize(&mut self) {
        // TODO: normalize system with slack
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        let relations = match self {
            LinearSystem::Single(relation) => {
                relation.remap(old_to_new);
                return;
            }
            LinearSystem::Conjunction(relations) => relations,
            LinearSystem::Disjunction(relations) => relations,
        };

        for relation in relations {
            relation.remap(old_to_new);
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.relations_iter()
            .flat_map(|relation| relation.used_ids())
            .collect()
    }

    fn relations_iter(&self) -> Box<dyn Iterator<Item = &LinearRelation> + '_> {
        match self {
            LinearSystem::Single(relation) => Box::new(std::iter::once(relation)),
            LinearSystem::Conjunction(relations) => Box::new(relations.iter()),
            LinearSystem::Disjunction(relations) => Box::new(relations.iter()),
        }
    }

    fn into_relations_iter(self) -> Box<dyn Iterator<Item = LinearRelation>> {
        match self {
            LinearSystem::Single(relation) => Box::new(std::iter::once(relation)),
            LinearSystem::Conjunction(relations) => Box::new(relations.into_iter()),
            LinearSystem::Disjunction(relations) => Box::new(relations.into_iter()),
        }
    }

    pub fn try_into_polynomial(self) -> Result<LinearPolynomial, Self> {
        let LinearSystem::Single(relation) = self else {
            return Err(self);
        };

        let bound = relation.polynomial().bound();

        match bound.width() {
            0 => {
                // can convert into empty polynomial
                Ok(LinearPolynomial::empty(bound))
            }
            1 => {
                // can convert into Boolean
                if relation.slack().is_nonzero() {
                    // this is always true
                    return Ok(LinearPolynomial::single_bit(true));
                }

                // the relation is left <= 0, i.e. left == 0
                // we must bit-not to obtain (!left) == (!1)
                // i.e. !left == 1, which can be converted to polynomial !left

                let bit_not_polynomial = relation.into_polynomial().bit_not();

                Ok(bit_not_polynomial)
            }
            _ => {
                let slack = *relation.slack();

                let Some((monomial, constant)) =
                    relation.polynomial().monomial_and_constant_value()
                else {
                    // cannot convert
                    return Err(LinearSystem::Single(relation));
                };

                if let Some(monomial) = monomial {
                    let slice = monomial.slice;
                    let coefficient = monomial.coefficient;

                    // if the monomial is single-bit, we will be able to simplify
                    if slice.width.get() != 1 {
                        return Err(LinearSystem::Single(relation));
                    }

                    let result_if_zero = constant.ule(slack);
                    let result_if_one = coefficient.add(constant).ule(slack);

                    if result_if_zero == result_if_one {
                        // tautology / contradiction
                        return Ok(LinearPolynomial::from_constant(result_if_one));
                    }

                    // if result_if_zero is 0 and result_if_one is 1, we want to construct single_bit
                    // if result_if_zero is 1 and result_if_one is 0, we want to construct (single_bit + 1) mod 2
                    let constant = result_if_zero;

                    let single_bit_bound = RBound::single_bit_bound();

                    let polynomial = LinearPolynomial::from_monomial_and_constant(
                        LinearMonomial::new(ConcreteBitvector::one(single_bit_bound), slice),
                        constant,
                    );

                    Ok(polynomial)
                } else {
                    // the result is whether constant <= slack
                    Ok(LinearPolynomial::from_constant(constant.ule(slack)))
                }
            }
        }
    }
}

impl Debug for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let universal = matches!(self, LinearSystem::Conjunction(_));
        let mut is_first = true;
        for relation in self.relations_iter() {
            if is_first {
                is_first = false;
            } else if universal {
                write!(f, " ∧ ")?;
            } else {
                write!(f, " ∨ ")?;
            }

            Debug::fmt(relation, f)?;
        }
        Ok(())
    }
}
