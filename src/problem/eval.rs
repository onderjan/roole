use std::{
    fmt::{Debug, Display, UpperHex},
    ops::ControlFlow,
};

use crate::{
    domain::{
        bitvector::{
            RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
        },
        traits::forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    },
    problem::{
        Problem,
        assignment::Assignment,
        domain::OperationDomain,
        formula::{FormulaId, VariableId},
        operation::{LinearPolynomial, OperationId},
    },
};

pub struct Evaluator<'a, D: EvaluableDomain> {
    problem: &'a Problem,
    // the results are indexed by OperationId
    results: Vec<Option<D>>,
}

impl<'a, D: EvaluableDomain> Evaluator<'a, D> {
    pub fn new(problem: &'a Problem) -> Self {
        Self {
            problem,
            results: vec![None; problem.operations.len()],
        }
    }

    pub fn problem(&self) -> &'a Problem {
        self.problem
    }

    pub fn result(&self, operation_id: OperationId) -> &D {
        self.results[operation_id.0]
            .as_ref()
            .expect("Result of operation {:?} should be computed")
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
        // must set previous results to None work with new assignment
        // keep the allocated vector for reuse
        for result in &mut self.results {
            *result = None;
        }

        let mut op_stack = vec![self.problem.assertion];
        let mut resolve = Vec::new();

        while let ControlFlow::Continue(()) =
            self.evaluate_formula(assignment, &mut op_stack, &mut resolve, preprocess)
        {}

        self.fetch_result(assignment, self.problem.assertion)
    }

    fn evaluate_formula(
        &mut self,
        assignment: &Assignment<D>,
        op_stack: &mut Vec<FormulaId>,
        resolve: &mut Vec<FormulaId>,
        preprocess: bool,
    ) -> ControlFlow<(), ()> {
        let Some(formula_id) = op_stack.pop() else {
            return ControlFlow::Break(());
        };

        let operation_id = match formula_id {
            FormulaId::Variable(_) => {
                // nothing to evaluate
                return ControlFlow::Continue(());
            }
            FormulaId::Operation(operation_id) => operation_id,
        };

        let operation = &self.problem.operations[operation_id.0];
        let dependencies = operation.used_ids();

        // resolve is empty here

        for dependency in dependencies.into_iter().rev() {
            assert_ne!(dependency, formula_id);
            if let FormulaId::Operation(dependency_operation_id) = dependency
                && self.results[dependency_operation_id.0].is_none()
            {
                resolve.push(dependency);
            }
        }

        if resolve.is_empty() {
            let evaluated =
                operation.evaluate(|formula_id| self.fetch_result(assignment, formula_id));
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

            // replace top with formula
            let evaluated = if evaluated == D::top(bound) {
                D::formula(formula_id, bound)
            } else {
                evaluated
            };

            self.results[operation_id.0] = Some(evaluated);
            // resolve is empty
        } else {
            // push the current formula id to operation stack before the dependencies
            // so the dependencies will get resolved before it is next encountered
            op_stack.push(formula_id);
            // append resolve to operation stack, empties it
            op_stack.append(resolve);
        }

        ControlFlow::Continue(())
    }

    fn fetch_result(&self, assignment: &Assignment<D>, formula_id: FormulaId) -> D {
        match formula_id {
            FormulaId::Variable(variable_id) => assignment.value(variable_id).clone(),
            FormulaId::Operation(operation_id) => self.results[operation_id.0]
                .as_ref()
                .expect("Fetched result of formula {:?} should be already computed")
                .clone(),
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        let mut franz = f.debug_struct("Evaluator");

        struct FieldStr<'a>(&'a str);
        impl Debug for FieldStr<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        for (variable_id, variable) in self.problem.variables.iter().enumerate() {
            let variable_id = VariableId(variable_id);
            franz.field(format!("{:?}", variable_id).as_str(), &variable);
        }

        for (operation_id, operation) in self.problem.operations.iter().enumerate() {
            let result = &self.results[operation_id];
            let operation_id = OperationId(operation_id);
            let name = format!("{:?}", operation_id);

            let mut value = if hex {
                format!("{:#X}", operation)
            } else {
                format!("{:?}", operation)
            };

            if let Some(result) = result {
                value = if hex {
                    format!("{} -> {:#X}", value, result)
                } else {
                    format!("{} -> {:?}", value, result)
                };
            }

            franz.field(&name, &FieldStr(&value));
        }

        franz.finish()
    }
}

pub trait EvaluableDomain:
    BitvectorDomain<Bound = RBound>
    + HwArith
    + Bitwise
    + TypedEq<Output = Self>
    + TypedCmp<Output = Self>
    + HwShift<Output = Self>
    + BExt<RBound, Output = Self>
    + Debug
    + UpperHex
{
    fn formula(formula: FormulaId, bound: RBound) -> Self;
}

impl EvaluableDomain for AbstractBitvector<RBound> {
    fn formula(formula: FormulaId, bound: RBound) -> Self {
        let _ = formula;
        Self::top(bound)
    }
}

impl EvaluableDomain for OperationDomain {
    fn formula(formula_id: FormulaId, bound: RBound) -> Self {
        OperationDomain::from_polynomial(LinearPolynomial::from_formula(formula_id, bound))
    }
}

impl<D: EvaluableDomain> Debug for Evaluator<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl<D: EvaluableDomain> UpperHex for Evaluator<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
