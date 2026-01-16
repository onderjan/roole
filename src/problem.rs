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
pub use eval::Evaluator;

/// A satisfiability problem.
#[derive(Debug)]
pub struct Problem {
    /// Widths of universally-quantified bitvector variables.
    variable_widths: Vec<u32>,
    /// Operations on the variables and results of other operations.
    operations: Vec<Operation>,
    /// Formula id of the variable/operation which serves as the assertion.
    ///
    /// Must have a single-bit result.
    ///
    /// The problem is satisfiable exactly if it evaluates to 1 with
    /// at least one variable assignment.
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

    /// An assignment of variables where all variables are unknown.
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
