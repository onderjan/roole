use std::fmt::Debug;

use itertools::Itertools;

use crate::{
    domain::{
        bitvector::{RBound, abstr::AbstractBitvector},
        traits::Join,
        value::ThreeValued,
    },
    problem::decision::Decision,
};

#[derive(Clone)]
pub struct Assignment {
    pub values: Vec<AbstractBitvector<RBound>>,
}

impl Assignment {
    pub fn contains(&self, other: &Assignment) -> bool {
        for (our_value, other_value) in self.values.iter().zip_eq(&other.values) {
            if !our_value.contains(other_value) {
                return false;
            }
        }
        true
    }

    pub fn join(mut self, other: &Assignment) -> Assignment {
        for (our_value, other_value) in self.values.iter_mut().zip_eq(&other.values) {
            *our_value = our_value.join(other_value);
        }

        self
    }

    pub fn volume(&self) -> u64 {
        let mut count = 0;

        for our_value in self.values.iter() {
            count += our_value.get_unknown_bits().to_u64().count_ones() as u64;
        }

        count
    }

    pub fn num_differences(&self, rhs: &Self) -> u64 {
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

    pub fn apply_decision_to_undecided(&mut self, decision: Decision, value: bool) {
        assert_eq!(
            self.values[decision.variable_index].three_valued_from_bit(decision.bit_index),
            ThreeValued::Unknown
        );

        self.values[decision.variable_index]
            .set_bit_to_three_valued(decision.bit_index, ThreeValued::from_bool(value));
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
