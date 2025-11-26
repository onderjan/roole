use std::fmt::Debug;

use crate::{
    check::clever::SearchSpace,
    formula::{FormulaId, Operation},
};

mod assignment;
mod clever;
mod eval;

use assignment::Assignment;

#[derive(Debug)]
pub struct Checker {
    variable_widths: Vec<u32>,
    operations: Vec<Operation>,
    assertion: FormulaId,
}

impl Checker {
    pub fn new(
        variable_widths: Vec<u32>,
        operations: Vec<Operation>,
        assertion: FormulaId,
    ) -> Self {
        eprintln!("Checking satisfiability");

        Self {
            variable_widths,
            operations,
            assertion,
        }
    }

    pub fn check(&self) {
        let mut space: SearchSpace<'_, clever::LinearLearned> = SearchSpace::new(self);

        //let result = checker.brute_force();
        let result = space.dpll();

        match result {
            Some(assignment) => eprintln!("Satisfiable: {:?}", assignment.values),
            None => eprintln!("Unsatisfiable"),
        }
    }
}
