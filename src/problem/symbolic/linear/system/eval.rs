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
        let mut result = if self.conjunction {
            // start with full mask
            ConcreteBitvector::new_umax(self.bound)
        } else {
            // start with zero mask
            ConcreteBitvector::new_umin(self.bound)
        };

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

    pub fn constant_value_assuming(&self, assumption: &Self) -> Option<ConcreteBitvector<RBound>> {
        if assumption.expressions.is_empty() {
            return self.constant_value();
        }

        if !assumption.conjunction {
            // disjunction, try every disjunct separately
            for disjunct in &assumption.expressions {
                let disjunct = LinearSystem::from_expression(disjunct.clone());
                assert!(disjunct.conjunction);
                if let Some(result) = self.constant_value_assuming(&disjunct) {
                    return Some(result);
                }
            }
            return None;
        }

        // assumption is a conjunction, start with full mask

        let assumptions = &assumption.expressions;
        let mut result = ConcreteBitvector::new_umax(self.bound);

        for expression in &self.expressions {
            let Some(expression_result) = expression.constant_value_assuming(assumptions) else {
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
