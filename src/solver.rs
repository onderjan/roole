use std::{collections::BTreeSet, path::PathBuf};

use crate::{
    SolverMode,
    domain::bitvector::abstr::linear::LinearBitvector,
    problem::{Evaluator, Problem, formula::FormulaId},
    solver::internal::{InternalSolver, roole::RooleLearned},
};

#[cfg(feature = "cadical")]
mod cadical;
mod internal;

pub fn solve(
    problem: &Problem,
    output_dir: Option<PathBuf>,
    solver_mode: SolverMode,
    preprocess: bool,
) {
    if preprocess {
        // preprocess
        let mut preprocessor = Evaluator::<LinearBitvector>::new(problem);

        let assignment = problem.linear_assignment();
        preprocessor.evaluate(&assignment);

        eprintln!("Preprocessor: {:#?}", preprocessor);

        let mut used_ids = BTreeSet::new();

        let mut stack = vec![problem.assertion()];

        while let Some(formula_id) = stack.pop() {
            if !used_ids.insert(formula_id) {
                continue;
            }

            match formula_id {
                FormulaId::Variable(_) => {
                    // nothing used by this
                }
                FormulaId::Operation(operation_id) => {
                    let result = preprocessor.result(operation_id);
                    let mut used_own_id = false;
                    for used_id in result.used_ids() {
                        if used_id != formula_id {
                            stack.push(used_id);
                        } else {
                            used_own_id = true;
                        }
                    }

                    if used_own_id {
                        let operation = problem.operation(operation_id);

                        for used_id in operation.used_ids() {
                            stack.push(used_id);
                        }
                    }
                }
            }
        }

        eprintln!("Used ids: {:?}", used_ids);

        /*let problem = Problem {
            variable_widths,
            operations,
            assertion,
        };*/
    }

    // process
    match solver_mode {
        SolverMode::Internal => {
            let solver: InternalSolver<'_, RooleLearned> = InternalSolver::new(problem, output_dir);
            solver.solve();
        }
        SolverMode::Cadical => {
            #[cfg(feature = "cadical")]
            {
                cadical::CadicalSolver::new(problem, output_dir).solve();
            };

            #[cfg(not(feature = "cadical"))]
            {
                panic!("CaDiCaL feature not enabled");
            };
        }
    }
}
