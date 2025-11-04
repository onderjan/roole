use core::f32;

use num::{BigUint, One, ToPrimitive, Zero};

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

        let next_variable_bound = self.assignments[next_decision.variable_index].bound();
        let next_bit_index_mask =
            ConcreteBitvector::from_masked_u64(1 << next_decision.bit_index, next_variable_bound);

        // assign zero
        self.assignments[next_decision.variable_index] =
            self.assignments[next_decision.variable_index].bit_and(
                ThreeValuedBitvector::from_concrete_value(next_bit_index_mask.bit_not()),
            );
        self.decisions.push(next_decision);
    }

    fn inc_decision(&mut self) -> bool {
        while let Some(decision) = self.decisions.last_mut() {
            let bound = self.assignments[decision.variable_index].bound();
            let bit_index_mask = ConcreteBitvector::from_masked_u64(1 << decision.bit_index, bound);

            if decision.is_true {
                // go back to unknown, pop
                let zero_value = self.assignments[decision.variable_index].bit_and(
                    ThreeValuedBitvector::from_concrete_value(bit_index_mask.bit_not()),
                );

                self.assignments[decision.variable_index] =
                    self.assignments[decision.variable_index].join(&zero_value);

                self.decisions.pop();
            } else {
                // assign true and return
                decision.is_true = true;

                self.assignments[decision.variable_index] = self.assignments
                    [decision.variable_index]
                    .bit_or(ThreeValuedBitvector::from_concrete_value(bit_index_mask));
                return true;
            }
        }

        // increment wrapped
        false
    }
}

enum PartResult {
    Sat,
    UnsatPart,
    Unknown,
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
            decisions: Vec::new(),
            total_width,
            num_leaves,
            num_nodes,
            opened_nodes: BigUint::zero(),
            closed_leaves: BigUint::zero(),
        };

        let satisfiable = loop {
            match self.dpll_eval(&mut space) {
                PartResult::Sat => {
                    break true;
                }
                PartResult::Unknown => {
                    space.push_decision();
                }
                PartResult::UnsatPart => {
                    if !space.inc_decision() {
                        break false;
                    }
                }
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

    fn dpll_eval(&self, space: &mut SearchSpace) -> PartResult {
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

        if let Some(concrete_result) = result.concrete_value() {
            if concrete_result.is_nonzero() {
                eprintln!("Satisfiable: {:?}", space.assignments);
                return PartResult::Sat;
            } else {
                // unsatisfiable branch
                space.closed_leaves +=
                    BigUint::one() << (space.total_width - decision_level as u64);

                //eprintln!("Unsatisfiable part: {:?}", space.assignments);

                return PartResult::UnsatPart;
            }
        };

        PartResult::Unknown
    }
}
