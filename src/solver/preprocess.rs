use std::collections::BTreeMap;

use bimap::BiBTreeMap;

use crate::problem::{
    Evaluator, LinearBitvector, Problem,
    formula::{FormulaId, Operation, OperationId, VariableId},
};

pub fn preprocess(problem: &Problem) -> Problem {
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

    let mut new_variables = Vec::new();
    let mut new_operations = Vec::new();

    for (old_id, new_operation) in used_ids_redone {
        let new_id = match old_id {
            FormulaId::Variable(variable_id) => {
                let variable = problem.variable(variable_id);
                new_variables.push(variable.clone());
                FormulaId::Variable(VariableId(new_variables.len() - 1))
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

    let new_problem = Problem::new(new_variables, new_operations, new_assertion);

    eprintln!("New problem: {:#?}", new_problem);

    new_problem
}
