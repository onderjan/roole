use std::collections::{BTreeMap, HashMap};

use crate::{
    problem::{
        Evaluator, Problem, SymbolicDomain,
        formula::{FormulaId, OperationId, VariableId, operation::Operation},
    },
    solver::SolverSettings,
};

pub fn preprocess(problem: &Problem, settings: &SolverSettings) -> Problem {
    eprintln!("Performing preprocessing");
    let mut evaluator = Evaluator::<SymbolicDomain>::new(problem);
    evaluator.evaluate_preprocess(&problem.linear_assignment());

    eprintln!("Preprocessing evaluator: ");
    if settings.hexadecimal {
        eprintln!("{:#X}", evaluator);
    } else {
        eprintln!("{:#?}", evaluator);
    }

    let used_formulas = used_formulas(problem, &evaluator);
    let redirects = redirects_to_equal(&used_formulas);
    let preprocessed = create_preprocessed(problem, used_formulas, redirects);

    eprintln!("Preprocessed problem: ");
    if settings.hexadecimal {
        eprintln!("{:#X}", preprocessed);
    } else {
        eprintln!("{:#?}", preprocessed);
    }

    preprocessed
}

fn used_formulas<'a>(
    problem: &Problem,
    evaluator: &'a Evaluator<'a, SymbolicDomain>,
) -> BTreeMap<FormulaId, Option<&'a SymbolicDomain>> {
    let mut used_formulas = BTreeMap::new();
    let mut stack = vec![problem.assertion()];

    while let Some(formula_id) = stack.pop() {
        if used_formulas.contains_key(&formula_id) {
            // this formula is already marked as used
            continue;
        }

        let operation_id = match formula_id {
            FormulaId::Variable(_) => {
                // no further formulas used by this variable, just insert it
                used_formulas.insert(formula_id, None);
                continue;
            }
            FormulaId::Operation(operation_id) => operation_id,
        };

        let result = evaluator.get_operation_result_ref(operation_id);

        if let Some(result) = result {
            // consider the used ids from the domain value
            stack.extend(result.used_ids());
        } else {
            // consider the used ids from the operation
            stack.extend(problem.operation(operation_id).used_ids());
        }

        used_formulas.insert(formula_id, result);
    }

    used_formulas
}

fn create_preprocessed(
    problem: &Problem,
    used_formulas: BTreeMap<FormulaId, Option<&SymbolicDomain>>,
    operation_redirects: BTreeMap<OperationId, OperationId>,
) -> Problem {
    let mut old_to_new = BTreeMap::<FormulaId, FormulaId>::new();

    let mut new_variables = Vec::new();
    let mut new_operations = Vec::new();

    for old_id in used_formulas.keys().copied() {
        let old_id = match old_id {
            FormulaId::Variable(variable_id) => {
                // add the variable to the new problem
                let variable = problem.variable(variable_id);
                new_variables.push(variable.clone());

                // insert to old-to-new
                let new_id = FormulaId::Variable(VariableId(new_variables.len() - 1));
                old_to_new.insert(old_id, new_id);
                continue;
            }
            FormulaId::Operation(operation_id) => operation_id,
        };

        // go through all redirects first
        let mut redirected_id = old_id;
        while let Some(redirect) = operation_redirects.get(&redirected_id) {
            redirected_id = *redirect;
        }

        if redirected_id != old_id {
            // if redirected, just put this into old-to-new map
            let new_id = *old_to_new
                .get(&redirected_id.formula_id())
                .expect("Redirected id should be in old-to-new map");

            old_to_new.insert(old_id.formula_id(), new_id);
            continue;
        }

        // non-redirected operation
        let new_operation = used_formulas
            .get(&old_id.formula_id())
            .expect("Redirected id should be used");

        let new_operation = if let Some(SymbolicDomain::Linear(new_operation)) = new_operation {
            // keep the new operation
            Operation::Linear(new_operation.clone())
        } else {
            // replace by the original operation
            problem.operation(old_id).clone()
        };

        // push the remapped operation
        new_operations.push(new_operation.remapped(&old_to_new));

        // insert to old-to-new map
        let new_id = OperationId(new_operations.len() - 1);
        old_to_new.insert(old_id.formula_id(), new_id.formula_id());
    }

    let new_assertion = *old_to_new
        .get(&problem.assertion())
        .expect("Assertion should be within new operations");

    Problem::new(new_variables, new_operations, new_assertion)
}

fn redirects_to_equal(
    used_formulas: &BTreeMap<FormulaId, Option<&SymbolicDomain>>,
) -> BTreeMap<OperationId, OperationId> {
    // for each set of operations that are equal, redirects the operations
    // with higher formula id to the one with the lowest formula id.
    let mut redirects = BTreeMap::new();
    let mut unique_operations = HashMap::new();
    for (formula_id, operation) in used_formulas.iter() {
        let (FormulaId::Operation(operation_id), Some(operation)) = (*formula_id, *operation)
        else {
            continue;
        };
        if let Some(unique_id) = unique_operations.get(operation).copied() {
            let FormulaId::Operation(unique_id) = unique_id else {
                panic!("Unique formula with equal operation should be an operation");
            };
            redirects.insert(operation_id, unique_id);
        } else {
            unique_operations.insert(operation.clone(), *formula_id);
        }
    }

    redirects
}
