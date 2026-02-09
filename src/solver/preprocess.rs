use std::collections::{BTreeMap, HashMap};

use crate::{
    problem::{
        Evaluator, OperationDomain, Problem,
        formula::{FormulaId, VariableId},
        operation::{Operation, OperationId},
    },
    solver::SolverSettings,
};

pub fn preprocess(problem: &Problem, settings: &SolverSettings) -> Problem {
    let mut evaluator = Evaluator::<OperationDomain>::new(problem);
    evaluator.evaluate(&problem.linear_assignment());

    eprintln!("Preprocessing evaluator: ");
    if settings.hexadecimal {
        eprintln!("{:#X}", evaluator);
    } else {
        eprintln!("{:#?}", evaluator);
    }

    let used_operations = used_operations(problem, &evaluator);

    let redirects = unique_redirects(&used_operations);

    let preprocessed = create_preprocessed(problem, used_operations, redirects);

    eprintln!("Preprocessed problem: ");
    if settings.hexadecimal {
        eprintln!("{:#X}", preprocessed);
    } else {
        eprintln!("{:#?}", preprocessed);
    }

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
    redirects: BTreeMap<FormulaId, FormulaId>,
) -> Problem {
    let mut old_to_new = BTreeMap::<FormulaId, FormulaId>::new();

    let mut new_variables = Vec::new();
    let mut new_operations = Vec::new();

    for old_id in used_operations.keys().copied() {
        // go through all redirects first
        let mut redirected_id = old_id;
        while let Some(redirect) = redirects.get(&redirected_id) {
            redirected_id = *redirect;
        }

        if redirected_id != old_id {
            // if redirected, just put this into old-to-new map
            let new_id = *old_to_new
                .get(&redirected_id)
                .expect("Redirected id should be in old-to-new map");

            old_to_new.insert(old_id, new_id);
            continue;
        }

        let new_operation = used_operations
            .get(&old_id)
            .expect("Redirected id should be used");

        let new_id = match old_id {
            FormulaId::Variable(variable_id) => {
                let variable = problem.variable(variable_id);
                new_variables.push(variable.clone());
                FormulaId::Variable(VariableId(new_variables.len() - 1))
            }
            FormulaId::Operation(operation_id) => {
                let operation = if let Some(new_operation) = new_operation {
                    match &new_operation {
                        OperationDomain::Top(_) => problem.operation(operation_id),
                        OperationDomain::Linear(linear) => &Operation::Linear(linear.clone()),
                    }
                } else {
                    problem.operation(operation_id)
                };

                let operation = operation.remapped(&old_to_new);

                new_operations.push(operation);
                FormulaId::Operation(OperationId(new_operations.len() - 1))
            }
        };
        old_to_new.insert(old_id, new_id);
    }

    let new_assertion = *old_to_new
        .get(&problem.assertion())
        .expect("Assertion should be within new operations");

    Problem::new(new_variables, new_operations, new_assertion)
}

fn unique_redirects(
    used_operations: &BTreeMap<FormulaId, Option<&OperationDomain>>,
) -> BTreeMap<FormulaId, FormulaId> {
    let mut redirects = BTreeMap::new();
    let mut unique_operations = HashMap::new();
    for (formula_id, operation) in used_operations.iter() {
        let Some(operation) = *operation else {
            continue;
        };
        if let Some(unique_id) = unique_operations.get(operation).copied() {
            redirects.insert(*formula_id, unique_id);
        } else {
            unique_operations.insert(operation.clone(), *formula_id);
        }
    }

    redirects
}
