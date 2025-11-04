use core::f32;

use indicatif::ProgressStyle;
use num::{BigUint, ToPrimitive};

use crate::{
    domain::bitvector::{RBound, abstr::AbstractBitvector},
    formula::{FormulaId, Operation},
};

mod brute;
mod clever;
mod eval;

#[derive(Debug)]
pub struct Checker {
    variable_widths: Vec<u32>,
    operations: Vec<Operation>,
    assertion: FormulaId,
    progress_bar: indicatif::ProgressBar,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    values: Vec<AbstractBitvector<RBound>>,
}

impl Checker {
    pub fn check(variable_widths: Vec<u32>, operations: Vec<Operation>, assertion: FormulaId) {
        eprintln!("Checking satisfiability");

        let progress_bar = indicatif::ProgressBar::new(PRECISION_CONST);
        progress_bar.set_style(
            ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}").unwrap(),
        );

        let checker = Self {
            variable_widths,
            operations,
            assertion,
            progress_bar,
        };

        //let result = checker.brute_force();
        let result = checker.dpll();

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
