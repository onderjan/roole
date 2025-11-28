use std::fmt::Debug;

use crate::domain::bitvector::{RBound, abstr::AbstractBitvector};
use formula::{FormulaId, Operation};

pub mod formula;
pub mod solution;

mod assignment;
mod decision;
mod eval;

pub use assignment::Assignment;
pub use decision::Decision;

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

    pub fn unknown_assignment(&self) -> Assignment {
        let mut assignment = Assignment { values: Vec::new() };
        for width in &self.variable_widths {
            assignment
                .values
                .push(AbstractBitvector::new_unknown(RBound::new(*width)));
        }

        assignment
    }
}
