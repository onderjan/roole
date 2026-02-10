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

use super::{LinearExpression, LinearPolynomial};

mod arith;
mod bitwise;
mod cmp;
mod eq;
mod ext;
mod shift;
mod support;

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

    pub fn from_concrete(constant: ConcreteBitvector<RBound>) -> Self {
        Self::from_polynomial(LinearPolynomial::from_concrete(constant))
    }

    pub fn from_bool(value: bool) -> Self {
        Self::from_polynomial(LinearPolynomial::from_bool(value))
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        Self::from_polynomial(LinearPolynomial::from_formula(formula_id, bound))
    }

    fn from_expression(expression: LinearExpression) -> Self {
        let expression = expression.into_normal_form();
        Self {
            conjunction: true,
            bound: expression.bound(),
            expressions: vec![expression],
        }
    }

    fn from_polynomial(polynomial: LinearPolynomial) -> Self {
        Self::from_expression(LinearExpression::Polynomial(polynomial))
    }

    fn try_into_expression(self) -> Result<LinearExpression, LinearSystem> {
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

    pub fn ite(condition: Self, then_branch: Self, else_branch: Self) -> Result<Self, ()> {
        // try to convert to a system if all are expressions
        let (Ok(condition), Ok(then_branch), Ok(else_branch)) = (
            condition.try_into_expression(),
            then_branch.try_into_expression(),
            else_branch.try_into_expression(),
        ) else {
            return Err(());
        };

        LinearExpression::ite(condition, then_branch, else_branch)
            .map(LinearSystem::from_expression)
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
