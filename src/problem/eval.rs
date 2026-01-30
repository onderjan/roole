use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    ops::ControlFlow,
};

use crate::{
    domain::{
        bitvector::{
            RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
            concr::ConcreteBitvector,
        },
        traits::forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    },
    problem::{
        Problem,
        assignment::Assignment,
        domain::OperationDomain,
        formula::{FormulaId, VariableId},
        operation::{LinearCombination, OperationId},
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
        // must set previous results to None work with new assignment
        // keep the allocated vector for reuse
        for result in &mut self.results {
            *result = None;
        }

        let mut op_stack = vec![self.problem.assertion];
        let mut resolve = Vec::new();

        while let ControlFlow::Continue(()) =
            self.evaluate_formula(assignment, &mut op_stack, &mut resolve)
        {}

        self.fetch_result(assignment, self.problem.assertion)
    }

    fn evaluate_formula(
        &mut self,
        assignment: &Assignment<D>,
        op_stack: &mut Vec<FormulaId>,
        resolve: &mut Vec<FormulaId>,
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
            // replace top with formula
            let evaluated = if evaluated == D::top(bound) {
                D::formula(bound, formula_id)
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
{
    fn formula(bound: RBound, formula: FormulaId) -> Self;
}

impl EvaluableDomain for AbstractBitvector<RBound> {
    fn formula(bound: RBound, formula: FormulaId) -> Self {
        let _ = formula;
        Self::top(bound)
    }
}

impl EvaluableDomain for OperationDomain {
    fn formula(bound: RBound, formula: FormulaId) -> Self {
        let mut monomials = BTreeMap::new();
        monomials.insert(formula, ConcreteBitvector::one(bound));
        OperationDomain::from_combination(LinearCombination::new(
            ConcreteBitvector::zero(bound),
            monomials,
        ))
    }
}

impl<D: EvaluableDomain> Debug for Evaluator<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut franz = f.debug_struct("Evaluator");

        struct FieldStr<'a>(&'a str);

        impl Debug for FieldStr<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        for (variable_id, width) in self.problem.variables.iter().enumerate() {
            let variable_id = VariableId(variable_id);
            franz.field(
                format!("{:?}", variable_id).as_str(),
                &FieldStr(&format!("Bitvec_{:?}", width)),
            );
        }

        for (operation_id, operation) in self.problem.operations.iter().enumerate() {
            let result = &self.results[operation_id];
            let operation_id = OperationId(operation_id);
            let name = format!("{:?} = {:?}", operation_id, operation);

            if let Some(result) = result {
                franz.field(&name, result);
            } else {
                franz.field(&name, &FieldStr("⊥"));
            }
        }

        franz.finish()
    }
}
