use std::{collections::BTreeSet, fmt::Debug, num::NonZeroUsize};

use crate::{
    domain::bitvector::RBound,
    problem::{
        Problem,
        assignment::Assignment,
        formula::{FormulaId, OperationId},
    },
};

mod domain;
mod format;

pub use domain::EvaluableDomain;
use itertools::Itertools;

pub struct Evaluator<'a, D: EvaluableDomain> {
    problem: &'a Problem,
    // the uses are indexed by OperationId
    num_uses: Vec<usize>,
    // the results are indexed by OperationId
    results: Vec<Option<EvaluatorResult<D>>>,
}

#[derive(Clone, Debug)]
struct EvaluatorResult<D: EvaluableDomain> {
    value: D,
    remaining_uses: NonZeroUsize,
}

impl<'a, D: EvaluableDomain> Evaluator<'a, D> {
    pub fn new(problem: &'a Problem) -> Self {
        let num_uses = Self::compute_num_uses(problem);
        Self {
            problem,
            num_uses,
            results: vec![None; problem.operations.len()],
        }
    }

    pub fn problem(&self) -> &'a Problem {
        self.problem
    }

    /// Evaluates this problem assertion on the given variable assignment.
    ///
    /// The assignment structure must correspond to the problem variables.
    pub fn evaluate(&mut self, assignment: &Assignment<D>) -> D {
        self.evaluate_inner(assignment, false)
    }

    pub(crate) fn evaluate_preprocess(&mut self, assignment: &Assignment<D>) -> D {
        self.evaluate_inner(assignment, true)
    }

    fn evaluate_inner(&mut self, assignment: &Assignment<D>, preprocess: bool) -> D {
        let Some(assertion_id) = self.problem.assertion.operation_id() else {
            // assertion is on an operation, no need to evaluate anything
            return self.result(assignment, self.problem.assertion);
        };

        // must set previous results to None work with new assignment
        // keep the allocated vector for reuse
        for result in &mut self.results {
            *result = None;
        }

        let mut op_stack = vec![assertion_id];
        let mut resolve = Vec::new();

        while let Some(operation_id) = op_stack.pop() {
            self.evaluate_operation(
                assignment,
                operation_id,
                &mut op_stack,
                &mut resolve,
                preprocess,
            );
        }

        self.result(assignment, self.problem.assertion)
    }

    fn evaluate_operation(
        &mut self,
        assignment: &Assignment<D>,
        operation_id: OperationId,
        op_stack: &mut Vec<OperationId>,
        resolve: &mut Vec<OperationId>,
        preprocess: bool,
    ) {
        let operation = &self.problem.operations[operation_id.0];
        let operation_used_ids = operation.used_ids();

        // resolve is empty here

        for operation_dependency in operation_used_ids
            .iter()
            .rev()
            .cloned()
            .filter_map(FormulaId::operation_id)
        {
            if self.results[operation_dependency.0].is_none() {
                resolve.push(operation_dependency);
            }
        }

        if !resolve.is_empty() {
            // push the current operation to operation stack before the dependencies
            // so the dependencies will get resolved before it is next encountered
            op_stack.push(operation_id);
            // append resolve to operation stack, empties it
            op_stack.append(resolve);
            return;
        }

        let remaining_uses = self.num_uses[operation_id.0];
        let Some(remaining_uses) = NonZeroUsize::new(remaining_uses) else {
            return;
        };

        let evaluated = operation.evaluate(|formula_id| self.fetch_result(assignment, formula_id));
        let bound = evaluated.bound();

        // for debugging, we can disqualify some operations from preprocessing
        // this is useful for tracking bugs in preprocessing
        let _ = preprocess;
        /*
        let evaluated = if preprocess && operation_id.0 > XYZ {
            D::top(bound)
        } else {
            evaluated
        };
        */

        // if top value, do not store the result
        if evaluated == D::top(bound) {
            return;
        }

        // update remaining uses
        self.update_remaining_uses(operation_id, &evaluated, operation_used_ids);

        self.results[operation_id.0] = Some(EvaluatorResult {
            value: evaluated,
            remaining_uses,
        });
    }

    fn update_remaining_uses(
        &mut self,
        operation_id: OperationId,
        evaluated: &D,
        operation_used_ids: Vec<FormulaId>,
    ) {
        let operation_used_set = BTreeSet::from_iter(
            operation_used_ids
                .into_iter()
                .filter_map(FormulaId::operation_id),
        );
        let domain_used_set = BTreeSet::from_iter(
            evaluated
                .used_ids()
                .into_iter()
                .filter_map(FormulaId::operation_id),
        );

        assert!(!domain_used_set.contains(&operation_id));

        for newly_used in domain_used_set.difference(&operation_used_set) {
            if let Some(result) = &self.results[newly_used.0] {
                result
                    .remaining_uses
                    .checked_add(1)
                    .expect("Remaining uses should not overflow");
            }
        }

        for no_longer_used in operation_used_set.difference(&domain_used_set) {
            if let Some(result) = self.results[no_longer_used.0].as_mut() {
                if let Some(remaining_uses) = NonZeroUsize::new(result.remaining_uses.get() - 1) {
                    // we still retain the result, update the value of remaining uses
                    result.remaining_uses = remaining_uses;
                } else {
                    // drop the result
                    self.results[no_longer_used.0] = None;
                }
            }
        }
    }

    fn result(&self, assignment: &Assignment<D>, formula_id: FormulaId) -> D {
        match formula_id {
            FormulaId::Variable(variable_id) => assignment.value(variable_id).clone(),
            FormulaId::Operation(operation_id) => {
                if let Some(result) = self.get_operation_result_ref(operation_id) {
                    result.clone()
                } else {
                    // return top
                    let bound = RBound::new(self.problem.operation(operation_id).result_width());
                    D::top(bound)
                }
            }
        }
    }

    pub fn get_operation_result_ref(&self, operation_id: OperationId) -> Option<&D> {
        self.results[operation_id.0]
            .as_ref()
            .map(|result| &result.value)
    }

    fn fetch_result(&self, assignment: &Assignment<D>, formula_id: FormulaId) -> D {
        match formula_id {
            FormulaId::Variable(variable_id) => assignment.value(variable_id).clone(),
            FormulaId::Operation(operation_id) => self.results[operation_id.0]
                .as_ref()
                .expect("Fetched result of formula {:?} should be already computed")
                .value
                .clone(),
        }
    }

    fn compute_num_uses(problem: &Problem) -> Vec<usize> {
        let mut num_uses = vec![0; problem.operations.len()];

        for operation in problem.operations.iter() {
            // ensure the used ids are unique for a proper count
            for used_id in operation.used_ids().into_iter().unique() {
                if let Some(operation_id) = used_id.operation_id() {
                    num_uses[operation_id.0] += 1;
                }
            }
        }

        if let Some(operation_id) = problem.assertion.operation_id() {
            num_uses[operation_id.0] += 1;
        }

        num_uses
    }
}
