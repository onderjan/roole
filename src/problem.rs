use std::fmt::{Debug, Display, UpperHex};

use crate::{
    domain::{
        bitvector::{
            RBound,
            abstr::{BitvectorDomain, RBitvector},
        },
        value::ThreeValued,
    },
    problem::{
        eval::EvaluableDomain,
        formula::{FormulaId, OperationId, Variable, VariableId, operation::Operation},
    },
};

pub mod formula;
pub mod solution;

mod assignment;
mod decision;
mod eval;
mod symbolic;

pub use assignment::Assignment;
pub use decision::Decision;
pub use eval::Evaluator;
pub use symbolic::SymbolicDomain;

/// A satisfiability problem.
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

    pub fn trivial_result(&self) -> ThreeValued {
        let FormulaId::Operation(operation_id) = self.assertion else {
            return ThreeValued::Unknown;
        };

        match self.operation(operation_id) {
            Operation::Constant(value, width) => {
                assert_eq!(*width, 1);
                ThreeValued::from_bool(*value != 0)
            }
            _ => ThreeValued::Unknown,
        }
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

    pub fn linear_assignment(&self) -> Assignment<SymbolicDomain> {
        let mut assignment = Assignment { values: Vec::new() };
        for (variable_id, variable) in self.variables.iter().enumerate() {
            let bound = RBound::new(variable.width);

            assignment.values.push(SymbolicDomain::formula(
                FormulaId::Variable(VariableId(variable_id)),
                bound,
            ));
        }

        assignment
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        let mut franz = f.debug_struct("Problem");

        struct FieldStr<'a>(&'a str);
        impl Debug for FieldStr<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        for (variable_id, variable) in self.variables.iter().enumerate() {
            let variable_id = VariableId(variable_id);
            franz.field(format!("{:?}", variable_id).as_str(), &variable);
        }

        for (operation_id, operation) in self.operations.iter().enumerate() {
            let operation_id = OperationId(operation_id);
            let name = format!("{:?}", operation_id);

            let value = if hex {
                format!("{:#X}", operation)
            } else {
                format!("{:?}", operation)
            };

            franz.field(&name, &FieldStr(&value));
        }

        franz.finish()
    }
}

impl Debug for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
