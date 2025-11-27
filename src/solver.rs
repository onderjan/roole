use std::{fs::File, io::BufWriter, ops::ControlFlow};

use crate::{
    assignment::Assignment,
    domain::{bitvector::abstr::BitvectorDomain, value::ThreeValued},
    problem::Problem,
    solver::{
        partition::{Partition, ValueType},
        roole::RooleLearned,
    },
};
use stats::Stats;

mod learned;
mod partition;
mod stats;

pub use learned::*;

pub fn solve(problem: &Problem) {
    let mut solver: Solver<'_, RooleLearned> = Solver::new(problem);
    let result = solver.dpll();

    if let Some(result) = result {
        eprintln!("Satisfiable: {:?}", result);
    } else {
        eprintln!("Unsatisfiable");
    }
}

#[derive(Clone, Copy)]
struct Decision {
    pub variable_index: usize,
    pub bit_index: u32,
}

struct Solver<'a, L: Learned> {
    problem: &'a Problem,

    partition: Partition,
    learned: L,

    stats: Stats,
}

impl<'a, L: Learned> Solver<'a, L> {
    pub fn new(problem: &'a Problem) -> Self {
        let stats = Stats::new(problem);
        let partition = Partition::new(problem.variable_widths());
        let learned = Learned::new();

        Self {
            problem,
            partition,
            learned,
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
        // update progress stats

        self.stats.inc_opened_nodes();

        let decision_level = self.partition.decision_level();
        if decision_level < 12 {
            self.stats.update_progress_bar();
        }

        // consider the current value in the partition graph
        if let Some(current_value) = self.partition.current_value() {
            // the current value is already known, no need to look into learned/evaluate
            // should be false as true would break immediately
            assert!(!current_value);
            self.stats.inc_already_resolved();
        } else if self.learned.contains(self.partition.assignment()) {
            // the current assignment is contained by an assignment known to be false
            // so we know this one is false as well
            self.partition.set_current_value(false, ValueType::Learned);
            self.stats.inc_already_learned();
        } else {
            // we have to evaluate the assignment
            let result = self.problem.eval(self.partition.assignment());

            let Some(concrete_result) = result.concrete_value() else {
                // unknown result, we need to choose another decision
                // choose the decision with false phase
                self.partition.choose_decision(false);
                return ControlFlow::Continue(());
            };
            if concrete_result.is_nonzero() {
                // true result, i.e. satisfiable by the current assignment
                // break immediately
                self.partition.set_current_value(true, ValueType::Normal);
                return ControlFlow::Break(true);
            }

            // false result, i.e. not satisfiable by the current assignment
            self.partition.set_current_value(false, ValueType::Normal);

            // learn this false result
            self.learn();
        };

        // try non-chronological backtracking
        if self.backtrack() {
            return ControlFlow::Continue(());
        }

        // not possible to backtrack
        // increment decision
        self.stats
            .add_closed_leaves(self.stats.total_width() - decision_level);
        if !self.partition.inc_decision() {
            // we have explored the whole partition graph without satisfaction success
            // this means the formula is unsatisfiable
            return ControlFlow::Break(false);
        }

        // continue with the next decision
        ControlFlow::Continue(())
    }

    fn learn(&mut self) {
        let mut learning_assignment = self.partition.assignment().clone();

        for (decision, _phase, _uses_backtracking) in self.partition.rev_decision_iter() {
            // make decision bit unknown
            let original = learning_assignment.values[decision.variable_index];
            learning_assignment.values[decision.variable_index]
                .set_bit_to_three_valued(decision.bit_index, ThreeValued::Unknown);

            // evaluate
            let result = self.problem.eval(&learning_assignment);

            if let Some(concrete_value) = result.concrete_value() {
                assert!(concrete_value.is_zero());
            } else {
                // go back
                learning_assignment.values[decision.variable_index] = original;
            }
        }

        self.learned.add(learning_assignment);
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

                let result = self.problem.eval(&backtrack_assignment);

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

        self.stats.inc_backtrackings();

        true
    }
}
