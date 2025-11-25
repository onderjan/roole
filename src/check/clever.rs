use core::f32;
use num::{BigUint, One, ToPrimitive, Zero};
use std::{fs::File, io::BufWriter, ops::ControlFlow};

use crate::{
    check::{
        Assignment, PRECISION_CONST,
        clever::{
            learned::Learned,
            partition::{Partition, ValueType},
        },
        percent,
    },
    domain::{bitvector::abstr::BitvectorDomain, value::ThreeValued},
};

mod learned;
mod partition;

pub use learned::*;

struct SearchSpace<L: Learned> {
    partition: Partition,

    learning_assignment: Assignment,
    learned: L,

    total_width: u64,
    num_leaves: BigUint,
    num_nodes: BigUint,
    opened_nodes: BigUint,
    closed_leaves: BigUint,
    num_learned: usize,
}

impl super::Checker {
    pub fn dpll<L: Learned>(&self) -> Option<Assignment> {
        let total_width: u64 = self.variable_widths.iter().map(|width| *width as u64).sum();

        let num_leaves = BigUint::one() << total_width;
        let num_nodes = (num_leaves.clone() * 2u32) - 1u32;

        let mut space = SearchSpace::<L> {
            partition: Partition::new(&self.variable_widths),

            learned: Learned::new(),
            learning_assignment: Assignment { values: Vec::new() },

            total_width,
            num_leaves,
            num_nodes,
            opened_nodes: BigUint::zero(),
            closed_leaves: BigUint::zero(),
            num_learned: 0,
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
            space.num_learned,
        );

        let learned_file = File::create("learned.dot").expect("Learned file should be created");
        space
            .learned
            .write_dot(&mut BufWriter::new(learned_file))
            .expect("Learned file should be written");

        space.partition.write();

        /*for learned in space.learned_assignments {
            println!("{:?}", learned);
        }*/

        result
    }

    fn dpll_eval<L: Learned>(&self, space: &mut SearchSpace<L>) -> ControlFlow<bool> {
        //eprintln!("Eval assignment: {:?}", space.partition.assignment());
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

        if let Some(current_value) = space.partition.current_value() {
            assert!(!current_value);
        } else if space.learned.contains(space.partition.assignment()) {
            // already learned that this is false
            space.partition.set_current_value(false, ValueType::Learned);

            if self.backtrack(space) {
                return ControlFlow::Continue(());
            }
        } else {
            let result = self.eval_formula(space.partition.assignment(), self.assertion);

            let Some(concrete_result) = result.concrete_value() else {
                // unknown result, choose false decision
                space.partition.choose_decision(false);
                return ControlFlow::Continue(());
            };
            if concrete_result.is_nonzero() {
                // satisfiable with these decisions, break immediately
                space.partition.set_current_value(true, ValueType::Normal);
                return ControlFlow::Break(true);
            }

            // unsatisfiable with these decisions
            space.partition.set_current_value(false, ValueType::Normal);

            // learn
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

    fn learn<L: Learned>(&self, space: &mut SearchSpace<L>) {
        //eprintln!("Unsatisfiable part: {:?}", space.assignments);

        space
            .learning_assignment
            .clone_from(space.partition.assignment());

        for (decision, _phase, _uses_backtracking) in space.partition.rev_decision_iter() {
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
        space.num_learned += 1;
    }

    fn backtrack<L: Learned>(&self, space: &mut SearchSpace<L>) -> bool {
        // for backtracking, we will try to successively make unknown every decision except last

        let decision_level = space.partition.decision_level();

        let mut backtrack_assignment = space.partition.assignment().clone();

        let mut rev_decision_iter = space.partition.rev_decision_iter();

        let Some((_last_decision, last_phase, _uses_backtracking)) = rev_decision_iter.next()
        else {
            // no backtracking possible
            return false;
        };

        let mut num_inspected_levels = 0;
        let mut num_yoinked_levels = 0;

        for (decision, _phase, uses_backtracking) in rev_decision_iter {
            // undo decision
            backtrack_assignment.values[decision.variable_index]
                .set_bit_to_three_valued(decision.bit_index, ThreeValued::Unknown);

            if !space.learned.contains(&backtrack_assignment) {
                // we still may be able to salvage this by evaluating the formula

                let result = self.eval_formula(&backtrack_assignment, self.assertion);

                let Some(concrete_result) = result.concrete_value() else {
                    // unknown result, cannot backtrack anymore
                    break;
                };
                assert!(concrete_result.is_zero());
            }

            num_inspected_levels += 1;

            if !uses_backtracking {
                // levels up to this decision level should be yoinked
                num_yoinked_levels = num_inspected_levels;
            }
        }

        if num_yoinked_levels == 0 {
            // no backtracking to do
            return false;
        }

        // pop last decision
        space.partition.pop_decision();

        // yoink decisions that do not contribute
        for _ in 0..num_yoinked_levels {
            space.partition.pop_decision();
        }

        /*eprintln!(
            "Yoinking {} levels: {:?}",
            num_yoinked_levels,
            space.partition.assignment()
        );*/

        // force next decision
        space
            .partition
            .force_next_decision(decision_level, last_phase, false);

        //eprintln!("After forcing: {:?}", space.partition.assignment());

        true
    }
}
