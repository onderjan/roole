use core::f32;
use std::fmt::Debug;

use indicatif::ProgressStyle;
use itertools::Itertools;
use num::{BigUint, ToPrimitive};

use crate::{
    domain::{
        bitvector::{RBound, abstr::AbstractBitvector},
        traits::Join,
    },
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

#[derive(Clone)]
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
        let result = checker.dpll::<clever::LinearLearned>();

        match result {
            Some(assignment) => eprintln!("Satisfiable: {:?}", assignment.values),
            None => eprintln!("Unsatisfiable"),
        }
    }
}

impl Assignment {
    fn contains(&self, other: &Assignment) -> bool {
        for (our_value, other_value) in self.values.iter().zip_eq(&other.values) {
            if !our_value.contains(other_value) {
                return false;
            }
        }
        true
    }

    fn join(mut self, other: &Assignment) -> Assignment {
        for (our_value, other_value) in self.values.iter_mut().zip_eq(&other.values) {
            *our_value = our_value.join(other_value);
        }

        self
    }

    fn volume(&self) -> u64 {
        let mut count = 0;

        for our_value in self.values.iter() {
            count += our_value.get_unknown_bits().to_u64().count_ones() as u64;
        }

        count
    }

    fn num_differences(&self, rhs: &Self) -> u64 {
        let mut count = 0;

        for (our_value, rhs_value) in self.values.iter().zip_eq(rhs.values.iter()) {
            let our_zeros = our_value.get_possibly_zero_flags().to_u64();
            let our_ones = our_value.get_possibly_one_flags().to_u64();

            let rhs_zeros = rhs_value.get_possibly_zero_flags().to_u64();
            let rhs_ones = rhs_value.get_possibly_one_flags().to_u64();

            let zero_diff = our_zeros ^ rhs_zeros;
            let one_diff = our_ones ^ rhs_ones;
            let some_diff = zero_diff | one_diff;

            count += some_diff.count_ones() as u64;
        }

        count
    }
}

impl Debug for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        let mut first = true;
        for value in &self.values {
            if first {
                first = false;
            } else {
                write!(f, "_")?;
            }
            value.write_nonenclosed(f)?;
        }
        write!(f, "\"")
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
