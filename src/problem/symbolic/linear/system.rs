use std::{
    collections::BTreeMap,
    fmt::{Debug, UpperHex},
};

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{LinearExpression, LinearPolynomial, LinearRelation};

mod cmp;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearSystem {
    conjunction: bool,
    bound: RBound,
    expressions: Vec<LinearExpression>,
}

impl LinearSystem {
    pub fn bound(&self) -> RBound {
        self.bound
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let mut result = if self.conjunction {
            // start with full mask
            D::single_value(ConcreteBitvector::new_umax(self.bound))
        } else {
            // start with zero mask
            D::single_value(ConcreteBitvector::new_umin(self.bound))
        };

        for expression in &self.expressions {
            let expression_result = expression.evaluate(&fetch);
            if self.conjunction {
                result = result.bit_and(expression_result);
            } else {
                result = result.bit_or(expression_result);
            }
        }
        result
    }

    pub fn from_polynomial(polynomial: LinearPolynomial) -> Self {
        Self::from_expression(LinearExpression::Polynomial(polynomial))
    }

    pub fn from_relation(relation: LinearRelation) -> Self {
        Self::from_expression(LinearExpression::Relation(relation))
    }

    pub fn from_expression(expression: LinearExpression) -> Self {
        let expression = expression.into_normal_form();
        Self {
            conjunction: true,
            bound: expression.bound(),
            expressions: vec![expression],
        }
    }

    pub fn try_into_expression(self) -> Result<LinearExpression, LinearSystem> {
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

    pub fn try_into_polynomial(self) -> Result<LinearPolynomial, LinearSystem> {
        if self.expressions.len() != 1
            || !matches!(self.expressions[0], LinearExpression::Polynomial(_))
        {
            return Err(self);
        }

        let Ok(LinearExpression::Polynomial(polynomial)) =
            self.expressions.into_iter().exactly_one()
        else {
            panic!("Should be ensured to be a polynomial");
        };

        Ok(polynomial)
    }

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        if self.expressions.len() != 1 {
            return None;
        }

        self.expressions[0].constant_value()
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        let mut used_ids = Vec::new();
        for expression in &self.expressions {
            used_ids.extend(expression.used_ids());
        }

        used_ids
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        for expression in &mut self.expressions {
            expression.remap(old_to_new);
        }
    }

    pub fn bit_not(mut self) -> Self {
        self.conjunction = !self.conjunction;

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

    pub fn ite(condition: Self, then_branch: Self, else_branch: Self) -> Result<Self, ()> {
        // try to simplify with polynomial branches
        let (Ok(then_branch), Ok(else_branch)) = (
            then_branch.try_into_polynomial(),
            else_branch.try_into_polynomial(),
        ) else {
            return Err(());
        };

        // try to resolve all polynomials
        if let Ok(condition) = condition.try_into_polynomial()
            && let Ok(result) = LinearPolynomial::ite(condition, then_branch, else_branch)
        {
            return Ok(LinearSystem::from_polynomial(result));
        };

        Err(())
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
