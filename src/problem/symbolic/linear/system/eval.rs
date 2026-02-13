use std::collections::BTreeMap;

use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::Bitwise,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

use super::LinearSystem;

impl LinearSystem {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let mut result = D::single_value(ConcreteBitvector::new_bool_masked(
            self.conjunction,
            self.bound,
        ));

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
        let mut result = ConcreteBitvector::new_bool_masked(self.conjunction, self.bound);

        for expression in &self.expressions {
            let Some(expression_result) = expression.constant_value() else {
                // not a constant value
                return None;
            };
            if self.conjunction {
                result = result.bit_and(expression_result);
            } else {
                result = result.bit_or(expression_result);
            }
        }
        Some(result)
    }

    pub fn assume(&mut self, assumption: &Self) {
        if !assumption.conjunction && assumption.expressions.len() != 1 {
            // TODO: disjunction
            return;
        }

        // assumption system is a conjunction of assumptions

        for expression in &mut self.expressions {
            expression.assume(&assumption.expressions);
        }
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
