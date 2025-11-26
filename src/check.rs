use core::f32;
use std::fmt::Debug;

use indicatif::ProgressStyle;
use num::{BigUint, ToPrimitive};

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
    progress_bar: indicatif::ProgressBar,
}

impl Checker {
    pub fn new(
        variable_widths: Vec<u32>,
        operations: Vec<Operation>,
        assertion: FormulaId,
    ) -> Self {
        eprintln!("Checking satisfiability");

        let progress_bar = indicatif::ProgressBar::new(PRECISION_CONST);
        progress_bar.set_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}").unwrap(),
        );

        Self {
            variable_widths,
            operations,
            assertion,
            progress_bar,
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

const PRECISION_CONST: u64 = 1_000_000;

fn percent(dividend: &BigUint, divisor: &BigUint) -> f32 {
    (dividend.clone() * PRECISION_CONST / divisor.clone())
        .to_f32()
        .unwrap_or(f32::NAN)
        / (PRECISION_CONST as f32)
        * 100.
}
