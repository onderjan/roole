use std::fmt::Debug;

use crate::{
    domain::bitvector::{
        RBound,
        abstr::{BitvectorDomain, RBitvector},
    },
    problem::formula::{OperationId, Variable, VariableId},
};
use formula::{FormulaId, Operation};

pub mod formula;
pub mod solution;

mod assignment;
mod decision;
mod domain;
mod eval;

pub use assignment::Assignment;
pub use decision::Decision;
pub use domain::LinearBitvector;
pub use eval::Evaluator;

/// A satisfiability problem.
#[derive(Debug)]
pub struct Problem {
    /// Universally-quantified bitvector variables.
    variables: Vec<Variable>,
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
    pub fn new(variables: Vec<Variable>, operations: Vec<Operation>, assertion: FormulaId) -> Self {
        Self {
            variables,
            operations,
            assertion,
        }
    }

    pub fn variables(&self) -> &[Variable] {
        &self.variables
    }

    pub fn variable(&self, id: VariableId) -> &Variable {
        &self.variables[id.0]
    }

    pub fn operation(&self, id: OperationId) -> &Operation {
        &self.operations[id.0]
    }

    pub fn assertion(&self) -> FormulaId {
        self.assertion
    }

    /// An assignment of variables where all variables are unknown.
    pub fn unknown_assignment(&self) -> Assignment<RBitvector> {
        let mut assignment = Assignment { values: Vec::new() };
        for variable in &self.variables {
            assignment
                .values
                .push(RBitvector::top(RBound::new(variable.width)));
        }

        assignment
    }

    pub fn linear_assignment(&self) -> Assignment<LinearBitvector> {
        let mut assignment = Assignment { values: Vec::new() };
        for (variable_id, variable) in self.variables.iter().enumerate() {
            let bound = RBound::new(variable.width);
            assignment.values.push(LinearBitvector::for_formula_id(
                FormulaId::Variable(VariableId(variable_id)),
                bound,
            ));
        }

        assignment
    }
}
