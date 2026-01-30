use std::collections::BTreeMap;

use bimap::BiBTreeMap;

use crate::problem::{
    Evaluator, OperationDomain, Problem,
    formula::{FormulaId, Operation, OperationId, VariableId},
};

pub fn preprocess(problem: &Problem) -> Problem {
    let mut evaluator = Evaluator::<OperationDomain>::new(problem);
    evaluator.evaluate(&problem.linear_assignment());

    eprintln!("Preprocessing evaluator: {:#?}", evaluator);

    let used_operations = used_operations(problem, &evaluator);
    let preprocessed = create_preprocessed(problem, used_operations);

    eprintln!("Preprocessed problem: {:#?}", preprocessed);

    preprocessed
}

fn used_operations<'a>(
    problem: &Problem,
    evaluator: &'a Evaluator<'a, OperationDomain>,
) -> BTreeMap<FormulaId, Option<&'a OperationDomain>> {
    let mut used_operations = BTreeMap::new();
    let mut stack = vec![problem.assertion()];

    while let Some(formula_id) = stack.pop() {
        if used_operations.contains_key(&formula_id) {
            continue;
        }

        let redone = match formula_id {
            FormulaId::Variable(_) => {
                // nothing used by this
                None
            }
            FormulaId::Operation(operation_id) => {
                let result = evaluator.result(operation_id);
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

        used_operations.insert(formula_id, redone);
    }

    used_operations
}

fn create_preprocessed(
    problem: &Problem,
    used_operations: BTreeMap<FormulaId, Option<&OperationDomain>>,
) -> Problem {
    let mut old_to_new = BiBTreeMap::new();

    let mut new_variables = Vec::new();
    let mut new_operations = Vec::new();

    for (old_id, new_operation) in used_operations {
        let new_id = match old_id {
            FormulaId::Variable(variable_id) => {
                let variable = problem.variable(variable_id);
                new_variables.push(variable.clone());
                FormulaId::Variable(VariableId(new_variables.len() - 1))
            }
            FormulaId::Operation(operation_id) => {
                let operation = if let Some(new_operation) = new_operation {
                    match &new_operation {
                        OperationDomain::Top(_) => {
                            problem.operation(operation_id).remapped(&old_to_new)
                        }
                        OperationDomain::Combination(linear_combination) => {
                            Operation::LinearCombination(
                                linear_combination.clone().remap(&old_to_new),
                            )
                        }
                        OperationDomain::System(linear_system) => {
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

    Problem::new(new_variables, new_operations, new_assertion)
}
