use std::collections::BTreeMap;

use crate::{
    domain::bitvector::{RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

use super::LinearSystem;

impl LinearSystem {
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
}
