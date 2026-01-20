use std::fmt::Debug;

use itertools::Itertools;

use crate::{
    domain::{
        bitvector::{
            RBound,
            abstr::{BitvectorDomain, RBitvector},
        },
        value::ThreeValued,
    },
    problem::{decision::Decision, formula::VariableId},
};

/// Assignment of problem variables to abstract bitvector values.
#[derive(Clone)]
pub struct Assignment<D: BitvectorDomain<Bound = RBound>> {
    pub(super) values: Vec<D>,
}

impl<D: BitvectorDomain<Bound = RBound>> Assignment<D> {
    pub fn values(&self) -> &[D] {
        &self.values
    }

    pub fn value(&self, id: VariableId) -> &D {
        &self.values[id.0]
    }

    pub fn join(mut self, other: &Self) -> Self {
        for (our_value, other_value) in self.values.iter_mut().zip_eq(&other.values) {
            our_value.apply_join(other_value);
        }

        self
    }

    pub fn contains(&self, other: &Self) -> bool {
        for (our_value, other_value) in self.values.iter().zip_eq(&other.values) {
            if !our_value.contains(other_value) {
                return false;
            }
        }
        true
    }
}

impl Assignment<RBitvector> {
    pub fn set_decision_value(&mut self, decision: Decision, value: ThreeValued) {
        self.values[decision.variable_index()].set_bit_to_three_valued(decision.bit_index(), value);
    }

    pub fn apply_bool_decision_to_undecided(&mut self, decision: Decision, value: bool) {
        assert_eq!(self.get_decision_value(decision), ThreeValued::Unknown);
        self.set_decision_value(decision, ThreeValued::from_bool(value));
    }

    pub fn get_decision_value(&self, decision: Decision) -> ThreeValued {
        self.values[decision.variable_index()].three_valued_from_bit(decision.bit_index())
    }
}

impl Debug for Assignment<RBitvector> {
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
