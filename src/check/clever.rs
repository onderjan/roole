use core::f32;
use num::{BigUint, One, ToPrimitive, Zero};
use std::ops::ControlFlow;

use crate::{
    check::{PRECISION_CONST, percent},
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain, three_valued::ThreeValuedBitvector},
            concr::ConcreteBitvector,
        },
        traits::{Join, forward::Bitwise},
    },
};

#[derive(Debug, Clone, Copy)]
struct Decision {
    variable_index: usize,
    bit_index: u32,
    is_true: bool,
}

struct SearchSpace {
    assignments: Vec<AbstractBitvector<RBound>>,
    learning_assignments: Vec<AbstractBitvector<RBound>>,
    decisions: Vec<Decision>,

    total_width: u64,
    num_leaves: BigUint,
    num_nodes: BigUint,
    opened_nodes: BigUint,
    closed_leaves: BigUint,
}

impl SearchSpace {
    fn push_decision(&mut self) {
        let next_decision = if let Some(last_decision) = self.decisions.last() {
            let mut next_variable_index = last_decision.variable_index;
            let mut next_bit_index = last_decision.bit_index + 1;
            if next_bit_index
                >= self.assignments[last_decision.variable_index]
                    .bound()
                    .width()
            {
                next_bit_index = 0;
                next_variable_index += 1;
            }

            Decision {
                variable_index: next_variable_index,
                bit_index: next_bit_index,
                is_true: false,
            }
        } else {
            Decision {
                variable_index: 0,
                bit_index: 0,
                is_true: false,
            }
        };

        // assign zero
        from_unknown_to_zero(&mut self.assignments, next_decision);
        self.decisions.push(next_decision);
    }

    fn inc_decision(&mut self) -> bool {
        while let Some(decision) = self.decisions.last_mut() {
            if decision.is_true {
                // go back to unknown, pop
                from_one_to_unknown(&mut self.assignments, *decision);
                self.decisions.pop();
            } else {
                // assign true and return
                decision.is_true = true;

                from_zero_to_one(&mut self.assignments, *decision);
                return true;
            }
        }

        // increment wrapped
        false
    }
}

impl super::Checker {
    pub fn recursive_dpll(&self) {
        let mut total_width = 0u64;
        let mut assignments = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            assignments.push(AbstractBitvector::new_unknown(RBound::new(width)));
            total_width = total_width
                .checked_add(width as u64)
                .expect("Total width should be in u64");
        }

        let num_leaves = BigUint::one() << total_width;
        let num_nodes = (num_leaves.clone() * 2u32) - 1u32;

        let mut space = SearchSpace {
            assignments,
            learning_assignments: Vec::new(),
            decisions: Vec::new(),
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

        if !satisfiable {
            self.progress_bar.set_position(PRECISION_CONST);
            self.progress_bar.set_message("100.00%");
            self.progress_bar.finish();
            eprintln!("Unsatisfiable");
        }

        let percent_opened_nodes = percent(&space.opened_nodes, &space.num_nodes);
        let percent_closed_leaves = percent(&space.closed_leaves, &space.num_leaves);

        eprintln!(
            "Info: {} nodes, {} opened ({:.3}%); {} leaves, {} closed ({:.3}%)",
            space.num_nodes,
            space.opened_nodes,
            percent_opened_nodes,
            space.num_leaves,
            space.closed_leaves,
            percent_closed_leaves
        );
    }

    fn dpll_eval(&self, space: &mut SearchSpace) -> ControlFlow<bool> {
        //eprintln!("Eval assignments: {:?}", space.assignments);
        space.opened_nodes += 1u32;

        let decision_level = space.decisions.len();

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

        let result = self.eval_formula(&space.assignments, self.assertion);

        let Some(concrete_result) = result.concrete_value() else {
            // unknown result, just push another decision
            space.push_decision();
            return ControlFlow::Continue(());
        };
        if concrete_result.is_nonzero() {
            // satisfiable with these decisions, break immediately
            eprintln!("Satisfiable: {:?}", space.assignments);
            return ControlFlow::Break(true);
        }

        // unsatisfiable with these decisions, learn, increment decision and continue
        self.learn(space);

        space.closed_leaves += BigUint::one() << (space.total_width - decision_level as u64);
        if !space.inc_decision() {
            return ControlFlow::Break(false);
        }
        ControlFlow::Continue(())
    }

    fn learn(&self, space: &mut SearchSpace) {
        //eprintln!("Unsatisfiable part: {:?}", space.assignments);

        space.learning_assignments.clone_from(&space.assignments);

        let mut has_unnecessary_decisions = false;

        for decision in &space.decisions {
            // make decision bit unknown

            let bit_index_mask = bit_index_mask(&space.learning_assignments, *decision);
            let decision_assignment = &mut space.learning_assignments[decision.variable_index];
            let original = *decision_assignment;

            let zero_value = decision_assignment.bit_and(
                ThreeValuedBitvector::from_concrete_value(bit_index_mask.bit_not()),
            );
            let one_value = decision_assignment
                .bit_or(ThreeValuedBitvector::from_concrete_value(bit_index_mask));

            *decision_assignment = decision_assignment.join(&zero_value).join(&one_value);

            // evaluate
            let result = self.eval_formula(&space.learning_assignments, self.assertion);

            if let Some(concrete_value) = result.concrete_value() {
                assert!(concrete_value.is_zero());
                has_unnecessary_decisions = true;
            } else {
                // go back
                space.learning_assignments[decision.variable_index] = original;
            }
        }

        if has_unnecessary_decisions {
            eprintln!(
                "Unnecesary decisions\nfrom {:?}\ninto {:?}",
                space.assignments, space.learning_assignments
            );
        }
    }
}

fn from_unknown_to_zero(assignments: &mut [AbstractBitvector<RBound>], decision: Decision) {
    let bit_index_mask = bit_index_mask(assignments, decision);
    let decision_assignment = &mut assignments[decision.variable_index];

    *decision_assignment = decision_assignment.bit_and(ThreeValuedBitvector::from_concrete_value(
        bit_index_mask.bit_not(),
    ));
}

fn from_zero_to_one(assignments: &mut [AbstractBitvector<RBound>], decision: Decision) {
    let bit_index_mask = bit_index_mask(assignments, decision);
    let decision_assignment = &mut assignments[decision.variable_index];

    *decision_assignment =
        decision_assignment.bit_or(ThreeValuedBitvector::from_concrete_value(bit_index_mask));
}

fn from_one_to_unknown(assignments: &mut [AbstractBitvector<RBound>], decision: Decision) {
    let bit_index_mask = bit_index_mask(assignments, decision);
    let decision_assignment = &mut assignments[decision.variable_index];

    let zero_value = decision_assignment.bit_and(ThreeValuedBitvector::from_concrete_value(
        bit_index_mask.bit_not(),
    ));

    *decision_assignment = decision_assignment.join(&zero_value);
}

fn bit_index_mask(
    assignments: &[AbstractBitvector<RBound>],
    decision: Decision,
) -> ConcreteBitvector<RBound> {
    let next_variable_bound = assignments[decision.variable_index].bound();

    ConcreteBitvector::from_masked_u64(1 << decision.bit_index, next_variable_bound)
}
