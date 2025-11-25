use core::f32;
use num::{BigUint, One, ToPrimitive, Zero};
use std::ops::ControlFlow;

use crate::{
    check::{
        Assignment, PRECISION_CONST,
        clever::{learned::Learned, partition::Partition},
        percent,
    },
    domain::{
        bitvector::{
            RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
        },
        value::ThreeValued,
    },
};

mod learned;
mod partition;

struct SearchSpace {
    partition: Partition,

    learning_assignment: Assignment,
    learned: Learned,

    total_width: u64,
    num_leaves: BigUint,
    num_nodes: BigUint,
    opened_nodes: BigUint,
    closed_leaves: BigUint,
}

impl SearchSpace {}

impl super::Checker {
    pub fn dpll(&self) -> Option<Assignment> {
        let mut total_width = 0u64;
        let mut values = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            values.push(AbstractBitvector::new_unknown(RBound::new(width)));
            total_width = total_width
                .checked_add(width as u64)
                .expect("Total width should be in u64");
        }

        let num_leaves = BigUint::one() << total_width;
        let num_nodes = (num_leaves.clone() * 2u32) - 1u32;

        let mut space = SearchSpace {
            partition: Partition::new(Assignment { values }),

            learned: Learned::new(),
            learning_assignment: Assignment { values: Vec::new() },

            total_width,
            num_leaves,
            num_nodes,
            opened_nodes: BigUint::zero(),
            closed_leaves: BigUint::zero(),
        };

        let satisfiable = loop {
            match self.dpll_eval(&mut space) {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(satisfiable) => break satisfiable,
            }
        };

        let result = if satisfiable {
            Some(space.partition.assignment().clone())
        } else {
            self.progress_bar.set_position(PRECISION_CONST);
            self.progress_bar.set_message("100.00%");
            self.progress_bar.finish();
            None
        };

        let percent_opened_nodes = percent(&space.opened_nodes, &space.num_nodes);
        let percent_closed_leaves = percent(&space.closed_leaves, &space.num_leaves);

        eprintln!(
            "Info: {} nodes, {} opened ({:.3}%); {} leaves, {} closed ({:.3}%), learned: {}",
            space.num_nodes,
            space.opened_nodes,
            percent_opened_nodes,
            space.num_leaves,
            space.closed_leaves,
            percent_closed_leaves,
            space.learned.number(),
        );

        space.learned.write();
        space.partition.write();

        /*for learned in space.learned_assignments {
            println!("{:?}", learned);
        }*/

        result
    }

    fn dpll_eval(&self, space: &mut SearchSpace) -> ControlFlow<bool> {
        //eprintln!("Eval assignments: {:?}", space.assignments);
        space.opened_nodes += 1u32;

        let decision_level = space.partition.decision_level();

        if decision_level < 12 {
            // update progress bar
            let progress =
                (space.closed_leaves.clone() * PRECISION_CONST) / space.num_leaves.clone();

            let progress_ratio = progress.to_f32().unwrap_or(f32::NAN) / PRECISION_CONST as f32;
            let progress_percent = progress_ratio * 100.;

            self.progress_bar
                .set_position(progress.to_u64().unwrap_or(0));
            self.progress_bar
                .set_message(format!("{:.2}%", progress_percent));
        }

        // see if we have already learned this

        if space.learned.contains(space.partition.assignment()) {
            // part unsatisfiable

            //space.partitions.inner.push(space.assignment.clone());

            /*
            // backtrack by popping decisions until it is no longer contained within the learned clause
            let already_learned = already_learned.clone();

            //eprintln!("Ours: {:?}\nLear: {:?}", space.assignment, already_learned);

            if let Some(mut popped_decision) = space.pop_decision() {
                while already_learned.contains(&space.assignment) {
                    eprintln!("Backtracked successfully");
                    if let Some(decision) = space.pop_decision() {
                        popped_decision = decision;
                    } else {
                        break;
                    }
                }
                // push last back
                space.push_decision(popped_decision);
            }*/
        } else {
            let result = self.eval_formula(space.partition.assignment(), self.assertion);

            let Some(concrete_result) = result.concrete_value() else {
                // unknown result, just push another decision
                space.partition.push_zero_decision();
                return ControlFlow::Continue(());
            };
            if concrete_result.is_nonzero() {
                // satisfiable with these decisions, break immediately
                return ControlFlow::Break(true);
            }

            // unsatisfiable with these decisions, learn
            self.learn(space);
        };

        // increment decision and continue

        space.closed_leaves += BigUint::one() << (space.total_width - decision_level);
        if !space.partition.inc_decision() {
            // whole unsatisfiable
            return ControlFlow::Break(false);
        }
        ControlFlow::Continue(())
    }

    fn learn(&self, space: &mut SearchSpace) {
        //eprintln!("Unsatisfiable part: {:?}", space.assignments);

        space
            .learning_assignment
            .clone_from(space.partition.assignment());

        for decision in space.partition.rev_decision_iter() {
            // make decision bit unknown
            let original = space.learning_assignment.values[decision.variable_index];
            space.learning_assignment.values[decision.variable_index]
                .set_bit_to_three_valued(decision.bit_index, ThreeValued::Unknown);

            // evaluate
            let result = self.eval_formula(&space.learning_assignment, self.assertion);

            if let Some(concrete_value) = result.concrete_value() {
                assert!(concrete_value.is_zero());
            } else {
                // go back
                space.learning_assignment.values[decision.variable_index] = original;
            }
        }

        //assert!(!self.is_learned(space, &space.learning_assignment));

        //println!("Learned assignment {:?}", space.learning_assignment);

        /*eprintln!(
            "Unnecesary decisions\nfrom {:?}\ninto {:?}",
            space.assignment, space.learning_assignment
        );*/
        space.learned.add(&space.learning_assignment);
    }
}
