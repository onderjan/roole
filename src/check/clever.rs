use core::f32;

use num::{BigUint, One, ToPrimitive, Zero};

use crate::{
    check::{PRECISION_CONST, SearchSpaceInfo, percent},
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain, three_valued::ThreeValuedBitvector},
            concr::ConcreteBitvector,
        },
        traits::forward::Bitwise,
    },
};

impl super::Checker {
    pub fn recursive_dpll(&self) {
        let mut total_width = 0u128;
        let mut assignments = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            assignments.push(AbstractBitvector::new_unknown(RBound::new(width)));
            total_width = total_width
                .checked_add(width as u128)
                .expect("Total width should be in u128");
        }

        let num_leaves = BigUint::one() << total_width;
        let num_nodes = (num_leaves.clone() * 2u32) - 1u32;

        let mut info = SearchSpaceInfo {
            total_width,
            num_leaves,
            num_nodes,
            opened_nodes: BigUint::zero(),
            closed_leaves: BigUint::zero(),
        };

        let satisfiable = self.dpll_recursion(&mut info, &mut assignments, 0, 0, 0);

        if !satisfiable {
            self.progress_bar.set_position(PRECISION_CONST);
            self.progress_bar.set_message("100.00%");
            self.progress_bar.finish();
            eprintln!("Unsatisfiable");
        }

        let percent_opened_nodes = percent(&info.opened_nodes, &info.num_nodes);
        let percent_closed_leaves = percent(&info.closed_leaves, &info.num_leaves);

        eprintln!(
            "Info: {} nodes, {} opened ({:.3}%); {} leaves, {} closed ({:.3}%)",
            info.num_nodes,
            info.opened_nodes,
            percent_opened_nodes,
            info.num_leaves,
            info.closed_leaves,
            percent_closed_leaves
        );
    }

    fn dpll_recursion(
        &self,
        info: &mut SearchSpaceInfo,
        assignments: &mut [AbstractBitvector<RBound>],
        decision_level: u128,
        variable_index: usize,
        bit_index: u32,
    ) -> bool {
        info.opened_nodes += 1u32;

        if decision_level < 12 {
            // update progress bar
            let progress = (info.closed_leaves.clone() * PRECISION_CONST) / info.num_leaves.clone();

            let progress_ratio = progress.to_f32().unwrap_or(f32::NAN) / PRECISION_CONST as f32;
            let progress_percent = progress_ratio * 100.;

            self.progress_bar
                .set_position(progress.to_u64().unwrap_or(0));
            self.progress_bar
                .set_message(format!("{:.2}%", progress_percent));
        }

        let result = self.eval_formula(assignments, self.assertion);

        if let Some(concrete_result) = result.concrete_value() {
            if concrete_result.is_nonzero() {
                eprintln!("Satisfiable: {:?}", assignments);
                return true;
            } else {
                // unsatisfiable branch
                info.closed_leaves += BigUint::one() << (info.total_width - decision_level);

                return false;
            }
        };

        let original_value = assignments[variable_index];
        let bound = assignments[variable_index].bound();
        let bit_index_mask = ConcreteBitvector::from_masked_u64(1 << bit_index, bound);

        let next_decision_level = decision_level + 1;
        let mut next_variable_index = variable_index;
        let mut next_bit_index = bit_index + 1;
        if next_bit_index >= bound.width() {
            next_bit_index = 0;
            next_variable_index += 1;
        }

        // assign zero

        assignments[variable_index] = assignments[variable_index].bit_and(
            ThreeValuedBitvector::from_concrete_value(bit_index_mask.bit_not()),
        );

        if self.dpll_recursion(
            info,
            assignments,
            next_decision_level,
            next_variable_index,
            next_bit_index,
        ) {
            return true;
        }

        // assign one

        assignments[variable_index] = assignments[variable_index]
            .bit_or(ThreeValuedBitvector::from_concrete_value(bit_index_mask));

        if self.dpll_recursion(
            info,
            assignments,
            next_decision_level,
            next_variable_index,
            next_bit_index,
        ) {
            return true;
        }

        // go back to unknown
        assignments[variable_index] = original_value;

        false
    }
}
