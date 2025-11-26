use std::fmt::Debug;

use crate::{
    assignment::Assignment,
    domain::bitvector::{RBound, abstr::AbstractBitvector},
    formula::{FormulaId, Operation},
};

mod eval;

#[derive(Debug)]
pub struct Problem {
    variable_widths: Vec<u32>,
    operations: Vec<Operation>,
    assertion: FormulaId,
}

impl Problem {
    pub fn new(
        variable_widths: Vec<u32>,
        operations: Vec<Operation>,
        assertion: FormulaId,
    ) -> Self {
        Self {
            variable_widths,
            operations,
            assertion,
        }
    }

    pub fn variable_widths(&self) -> &[u32] {
        &self.variable_widths
    }

    pub fn eval(&self, assignment: &Assignment) -> AbstractBitvector<RBound> {
        self.eval_formula(assignment, self.assertion)
    }
}
