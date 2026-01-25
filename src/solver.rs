use std::{collections::BTreeMap, path::PathBuf};

use bimap::BiBTreeMap;

use crate::{
    SolverMode,
    domain::bitvector::abstr::linear::LinearBitvector,
    problem::{
        Evaluator, Problem,
        formula::{FormulaId, Operation, OperationId, VariableId},
    },
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
    let new_problem = if preprocess {
        // preprocess
        let mut preprocessor = Evaluator::<LinearBitvector>::new(problem);

        let assignment = problem.linear_assignment();
        preprocessor.evaluate(&assignment);

        eprintln!("Preprocessor: {:#?}", preprocessor);

        let mut used_ids_redone = BTreeMap::new();

        let mut stack = vec![problem.assertion()];

        while let Some(formula_id) = stack.pop() {
            if used_ids_redone.contains_key(&formula_id) {
                continue;
            }

            let redone = match formula_id {
                FormulaId::Variable(_) => {
                    // nothing used by this
                    None
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
                        None
                    } else {
                        Some(result)
                    }
                }
            };

            used_ids_redone.insert(formula_id, redone);
        }

        eprintln!("Used ids: {:?}", used_ids_redone);

        let mut old_to_new = BiBTreeMap::new();

        let mut new_variable_widths = Vec::new();
        let mut new_operations = Vec::new();

        for (old_id, new_operation) in used_ids_redone {
            let new_id = match old_id {
                FormulaId::Variable(variable_id) => {
                    let width = problem.variable_width(variable_id);
                    new_variable_widths.push(width);
                    FormulaId::Variable(VariableId(new_variable_widths.len() - 1))
                }
                FormulaId::Operation(operation_id) => {
                    let operation = if let Some(new_operation) = new_operation {
                        match &new_operation {
                            LinearBitvector::Top(_) => {
                                problem.operation(operation_id).remapped(&old_to_new)
                            }
                            LinearBitvector::Combination(linear_combination) => {
                                Operation::LinearCombination(
                                    linear_combination.clone().remap(&old_to_new),
                                )
                            }
                            LinearBitvector::System(linear_system) => {
                                Operation::LinearSystem(linear_system.clone().remap(&old_to_new))
                            }
                        }
                    } else {
                        problem.operation(operation_id).remapped(&old_to_new)
                    };

                    new_operations.push(operation);
                    FormulaId::Operation(OperationId(new_operations.len() - 1))
                }
            };
            old_to_new.insert(old_id, new_id);
        }

        let new_assertion = *old_to_new
            .get_by_left(&problem.assertion())
            .expect("Assertion should be within new operations");

        let new_problem = Problem::new(new_variable_widths, new_operations, new_assertion);

        eprintln!("New problem: {:#?}", new_problem);

        Some(new_problem)
    } else {
        None
    };

    let problem = new_problem.as_ref().unwrap_or(problem);

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
