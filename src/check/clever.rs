use std::{fs::File, io::BufWriter, ops::ControlFlow};

use crate::{
    check::{
        Assignment, Checker,
        clever::{
            learned::Learned,
            partition::{Partition, ValueType},
        },
    },
    domain::{bitvector::abstr::BitvectorDomain, value::ThreeValued},
};
use stats::Stats;

mod learned;
mod partition;
mod stats;

pub use learned::*;

pub struct SearchSpace<'a, L: Learned> {
    checker: &'a Checker,
    partition: Partition,

    learning_assignment: Assignment,
    learned: L,

    stats: Stats,
}

impl<'a, L: Learned> SearchSpace<'a, L> {
    pub fn new(checker: &'a Checker) -> Self {
        let stats = Stats::new(checker);
        let partition = Partition::new(&checker.variable_widths);

        Self {
            checker,
            partition,
            learned: Learned::new(),
            learning_assignment: Assignment { values: Vec::new() },
            stats,
        }
    }

    pub fn dpll(&mut self) -> Option<Assignment> {
        let satisfiable = loop {
            match self.dpll_eval() {
                ControlFlow::Continue(()) => {}
                ControlFlow::Break(satisfiable) => break satisfiable,
            }
        };

        self.stats.finish();

        let learned_file = File::create("learned.dot").expect("Learned file should be created");
        self.learned
            .write_dot(&mut BufWriter::new(learned_file))
            .expect("Learned file should be written");

        self.partition.write();

        if satisfiable {
            Some(self.partition.assignment().clone())
        } else {
            None
        }
    }

    fn dpll_eval(&mut self) -> ControlFlow<bool> {
        self.stats.inc_opened_nodes();

        let decision_level = self.partition.decision_level();

        if decision_level < 12 {
            self.stats.update_progress_bar();
        }

        // see if we have already learned this

        if let Some(current_value) = self.partition.current_value() {
            assert!(!current_value);
        } else if self.learned.contains(self.partition.assignment()) {
            // already learned that this is false
            self.partition.set_current_value(false, ValueType::Learned);

            if self.backtrack() {
                return ControlFlow::Continue(());
            }
        } else {
            let result = self
                .checker
                .eval_formula(self.partition.assignment(), self.checker.assertion);

            let Some(concrete_result) = result.concrete_value() else {
                // unknown result, choose false decision
                self.partition.choose_decision(false);
                return ControlFlow::Continue(());
            };
            if concrete_result.is_nonzero() {
                // satisfiable with these decisions, break immediately
                self.partition.set_current_value(true, ValueType::Normal);
                return ControlFlow::Break(true);
            }

            // unsatisfiable with these decisions
            self.partition.set_current_value(false, ValueType::Normal);

            // learn
            self.learn();
        };

        // increment decision and continue

        self.stats
            .add_closed_leaves(self.stats.total_width() - decision_level);
        if !self.partition.inc_decision() {
            // whole unsatisfiable
            return ControlFlow::Break(false);
        }
        ControlFlow::Continue(())
    }

    fn learn(&mut self) {
        self.learning_assignment
            .clone_from(self.partition.assignment());

        for (decision, _phase, _uses_backtracking) in self.partition.rev_decision_iter() {
            // make decision bit unknown
            let original = self.learning_assignment.values[decision.variable_index];
            self.learning_assignment.values[decision.variable_index]
                .set_bit_to_three_valued(decision.bit_index, ThreeValued::Unknown);

            // evaluate
            let result = self
                .checker
                .eval_formula(&self.learning_assignment, self.checker.assertion);

            if let Some(concrete_value) = result.concrete_value() {
                assert!(concrete_value.is_zero());
            } else {
                // go back
                self.learning_assignment.values[decision.variable_index] = original;
            }
        }

        self.learned.add(&self.learning_assignment);
        self.stats.inc_learned();
    }

    fn backtrack(&mut self) -> bool {
        // for backtracking, we will try to successively make unknown every decision except last

        let decision_level = self.partition.decision_level();

        let mut backtrack_assignment = self.partition.assignment().clone();

        let mut rev_decision_iter = self.partition.rev_decision_iter();

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

            if !self.learned.contains(&backtrack_assignment) {
                // we still may be able to salvage this by evaluating the formula

                let result = self
                    .checker
                    .eval_formula(&backtrack_assignment, self.checker.assertion);

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
        self.partition.pop_decision();

        // yoink decisions that do not contribute
        for _ in 0..num_yoinked_levels {
            self.partition.pop_decision();
        }

        // force next decision
        self.partition
            .force_next_decision(decision_level, last_phase, false);

        true
    }
}
