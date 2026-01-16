use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use aws_smt_ir::visit::ControlFlow;
use cadical_sys::{CaDiCal, Status};

use crate::{
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        value::ThreeValued,
    },
    problem::{
        Assignment, Decision, Evaluator, Problem,
        solution::{Proof, Solution},
    },
};

pub struct CadicalSolver<'a> {
    evaluator: Evaluator<'a>,
    cadical: CaDiCal,

    assignment: Assignment,

    num_clauses: u32,
}

impl<'a> CadicalSolver<'a> {
    pub fn new(problem: &'a Problem, output_dir: Option<PathBuf>) -> Self {
        let assignment = problem.unknown_assignment();
        let _ = output_dir;

        Self {
            evaluator: Evaluator::new(problem),
            cadical: CaDiCal::new(),

            assignment,

            num_clauses: 0,
        }
    }

    pub fn solve(mut self) -> Solution {
        eprintln!("Solving");
        let progress_bar = indicatif::ProgressBar::new_spinner();

        progress_bar.set_style(
            indicatif::ProgressStyle::with_template("[{elapsed_precise}] {msg} {spinner}").unwrap(),
        );

        progress_bar.set_message(format!("{} clauses", self.num_clauses));

        let mut last_update_instant = Instant::now();

        let result = loop {
            match self.iteration() {
                ControlFlow::Continue(()) => {
                    let current_instant = Instant::now();
                    if current_instant - last_update_instant >= Duration::from_millis(10) {
                        progress_bar.tick();
                        progress_bar.set_message(format!("{} clauses", self.num_clauses));
                        last_update_instant = current_instant;
                    }
                }
                ControlFlow::Break(result) => break result,
            }
        };

        progress_bar.finish();

        match &result {
            Solution::Satisfiable(assignment) => {
                eprintln!("Satisfiable: {:?}", assignment);
            }
            Solution::Unsatisfiable(_proof) => {
                eprintln!("Unsatisfiable");
            }
        }

        result
    }

    pub fn iteration(&mut self) -> ControlFlow<Solution> {
        // Solve the problem
        let status = self.cadical.solve();
        match status {
            Status::SATISFIABLE => {}
            Status::UNSATISFIABLE => {
                // TODO extract proof
                let nodes = Vec::new();

                return ControlFlow::Break(Solution::Unsatisfiable(Proof::new(nodes)));
            }
            Status::UNKNOWN => panic!("Solution status unknown"),
        };

        for variable_index in 0..self.assignment.values().len() {
            for bit_index in 0..self.assignment.values()[variable_index].bound().width() {
                let cadical_index = 1 + variable_index as i32 * 64 + bit_index as i32;

                let value = if self.cadical.val(cadical_index) >= 0 {
                    ThreeValued::True
                } else {
                    ThreeValued::False
                };

                self.assignment
                    .set_decision_value(Decision::new(variable_index, bit_index), value);
            }
        }

        let eval_result = self.evaluator.evaluate(&self.assignment);
        assert_eq!(eval_result.bound().width(), 1);
        match eval_result.three_valued_from_bit(0) {
            ThreeValued::False => {
                // this part is unsatisfiable, we need to learn more for the SAT solver
            }
            ThreeValued::True => {
                // really satisfiable by this
                return ControlFlow::Break(Solution::Satisfiable(self.assignment.clone()));
            }
            ThreeValued::Unknown => {
                panic!("Fully set self.assignment should be satisfiable");
            }
        }

        // try to make the self.assignment unknown as much as we can

        for variable_index in (0..self.assignment.values().len()).rev() {
            for bit_index in (0..self.assignment.values()[variable_index].bound().width()).rev() {
                let decision = Decision::new(variable_index, bit_index);
                let decision_value = self.assignment.get_decision_value(decision);
                self.assignment
                    .set_decision_value(decision, ThreeValued::Unknown);

                let eval_result = self.evaluator.evaluate(&self.assignment);
                assert_eq!(eval_result.bound().width(), 1);
                match eval_result.three_valued_from_bit(0) {
                    ThreeValued::False => {
                        // assignment still unsatisfiable, continue iterating
                    }
                    ThreeValued::True => {
                        panic!(
                            "Should not make self.assignment satisfiable by setting it more unknown"
                        )
                    }
                    ThreeValued::Unknown => {
                        // no longer unsatisfiable, put decision value back
                        self.assignment.set_decision_value(decision, decision_value);
                    }
                }
            }
        }

        // learn the assignment as a new clause

        let mut clause = Vec::new();

        for variable_index in 0..self.assignment.values().len() {
            for bit_index in 0..self.assignment.values()[variable_index].bound().width() {
                let cadical_index = 1 + variable_index as i32 * 64 + bit_index as i32;

                let decision_value = self
                    .assignment
                    .get_decision_value(Decision::new(variable_index, bit_index));

                match decision_value {
                    ThreeValued::False => clause.push(cadical_index),
                    ThreeValued::True => clause.push(-cadical_index),
                    ThreeValued::Unknown => {
                        // do nothing
                    }
                }
            }
        }

        self.cadical.clause6(&clause);

        self.num_clauses += 1;

        ControlFlow::Continue(())
    }
}
