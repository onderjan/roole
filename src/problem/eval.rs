use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
};

use super::formula::{BiOp, BiOperator, ExtOp, FormulaId, IteOp, Operation, UniOp, UniOperator};
use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
            concr::ConcreteBitvector,
        },
        traits::forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    },
    problem::{
        Problem,
        assignment::Assignment,
        domain::{LinearBitvector, LinearCombination, LinearRelationType, LinearSystem},
        formula::{OperationId, VariableId},
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

        let mut op_stack = vec![(self.problem.assertion, false)];

        while let Some((formula_id, evaluated)) = op_stack.pop() {
            match formula_id {
                FormulaId::Variable(_) => {}
                FormulaId::Operation(operation_id) => {
                    let operation = &self.problem.operations[operation_id.0];
                    if evaluated {
                        let evaluated = self.evaluate_operation(assignment, operation);
                        let bound = evaluated.bound();
                        // replace top with formula
                        let evaluated = if evaluated == D::top(bound) {
                            D::formula(bound, formula_id)
                        } else {
                            evaluated
                        };

                        self.results[operation_id.0] = Some(evaluated);
                    } else {
                        let dependencies = operation.used_ids();

                        op_stack.push((formula_id, true));
                        for dependency in dependencies.into_iter().rev() {
                            op_stack.push((dependency, false));
                        }
                    }
                }
            };
        }

        self.fetch_result(assignment, self.problem.assertion)
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

    fn evaluate_operation(&self, assignment: &Assignment<D>, operation: &Operation) -> D {
        match operation {
            Operation::Constant(value, width) => {
                let concrete = ConcreteBitvector::new(*value, RBound::new(*width));
                D::single_value(concrete)
            }
            Operation::UniOp(UniOp {
                op,
                input_width: _,
                inner,
            }) => {
                let inner = self.fetch_result(assignment, *inner);
                match op {
                    UniOperator::Not => inner.bit_not(),
                }
            }
            Operation::BiOp(BiOp {
                op,
                input_width: _,
                left,
                right,
            }) => {
                let left = self.fetch_result(assignment, *left);
                let right = self.fetch_result(assignment, *right);

                match op {
                    BiOperator::Add => left.add(right),
                    BiOperator::Sub => left.sub(right),
                    BiOperator::Mul => left.mul(right),

                    BiOperator::BitAnd => left.bit_and(right),
                    BiOperator::BitOr => left.bit_or(right),
                    BiOperator::BitXor => left.bit_xor(right),

                    BiOperator::Eq => TypedEq::eq(left, right),
                    BiOperator::Ne => TypedEq::ne(left, right),
                    BiOperator::Implies => (left.bit_not()).bit_or(right),

                    BiOperator::Ult => TypedCmp::ult(left, right),
                    BiOperator::Ule => TypedCmp::ule(left, right),
                    BiOperator::Ugt => TypedCmp::ule(left, right).bit_not(),
                    BiOperator::Uge => TypedCmp::ult(left, right).bit_not(),

                    BiOperator::Slt => TypedCmp::slt(left, right),
                    BiOperator::Sle => TypedCmp::sle(left, right),
                    BiOperator::Sgt => TypedCmp::sle(left, right).bit_not(),
                    BiOperator::Sge => TypedCmp::slt(left, right).bit_not(),

                    BiOperator::Shl => left.logic_shl(right),
                    BiOperator::Lshr => left.logic_shr(right),
                    BiOperator::Ashr => left.arith_shr(right),
                }
            }
            Operation::ExtOp(ExtOp {
                signed,
                input_width: _,
                output_width,
                inner,
            }) => {
                let inner = self.fetch_result(assignment, *inner);
                let output_bound = RBound::new(*output_width);
                if *signed {
                    BExt::sext(inner, output_bound)
                } else {
                    BExt::uext(inner, output_bound)
                }
            }
            Operation::IteOp(IteOp {
                condition,
                width: _,
                formula_then,
                formula_else,
            }) => {
                let condition = self.fetch_result(assignment, *condition);
                assert_eq!(condition.bound().width(), 1);

                if let Some(condition_value) = condition.concrete_value() {
                    if condition_value.is_nonzero() {
                        // only then taken
                        self.fetch_result(assignment, *formula_then)
                    } else {
                        // only else taken
                        self.fetch_result(assignment, *formula_else)
                    }
                } else {
                    // both can be taken, join them
                    let value_then = self.fetch_result(assignment, *formula_then);
                    let value_else = self.fetch_result(assignment, *formula_else);
                    value_then.join(&value_else)
                }
            }
            Operation::ConcatOp(concat_op) => {
                let left = self.fetch_result(assignment, concat_op.left);
                let right = self.fetch_result(assignment, concat_op.right);

                assert_eq!(left.bound().width(), concat_op.left_width);
                assert_eq!(right.bound().width(), concat_op.right_width);

                let result_width = concat_op.left_width + concat_op.right_width;
                let result_bound = RBound::new(result_width);

                // zero-extend both to result width
                let left = left.uext(result_bound);
                let right = right.uext(result_bound);

                // shift left by right width
                let right_width_bitvector =
                    ConcreteBitvector::new(concat_op.right_width as u64, result_bound);
                let left = left.logic_shl(D::single_value(right_width_bitvector));

                // bit-or both
                left.bit_or(right)
            }
            Operation::ExtractOp(extract_op) => {
                let inner = self.fetch_result(assignment, extract_op.inner);

                assert!(inner.bound().width() >= extract_op.lsb + extract_op.width.get());

                // shift right by lsb
                // it should not matter which shift it is, perform it unsigned
                let concrete_rhs = ConcreteBitvector::new(extract_op.lsb.into(), inner.bound());
                let inner = inner.logic_shr(D::single_value(concrete_rhs));

                // narrow to extraction width
                inner.uext(RBound::new(extract_op.width.get()))
            }
            Operation::LinearCombination(combination) => {
                self.evaluate_combination(assignment, combination)
            }
            Operation::LinearSystem(system) => self.evaluate_system(assignment, system),
        }
    }

    fn evaluate_combination(
        &self,
        assignment: &Assignment<D>,
        combination: &LinearCombination,
    ) -> D {
        let mut value = D::single_value(combination.constant);
        for (formula_id, coeff) in &combination.coefficients {
            let formula_value = self.fetch_result(assignment, *formula_id);
            let term_value = formula_value.mul(D::single_value(*coeff));
            value = value.add(term_value);
        }

        value
    }

    fn evaluate_system(&self, assignment: &Assignment<D>, system: &LinearSystem) -> D {
        let bound = RBound::new(1);
        let mut result = if system.universal {
            // start with 1
            D::single_value(ConcreteBitvector::one(bound))
        } else {
            // start with 0
            D::single_value(ConcreteBitvector::zero(bound))
        };

        for relation in &system.relations {
            let combination = &relation.combination;
            let relation_result = match &relation.ty {
                LinearRelationType::Eq => {
                    let zero =
                        D::single_value(ConcreteBitvector::zero(combination.constant.bound()));
                    let value = self.evaluate_combination(assignment, combination);
                    TypedEq::eq(value, zero)
                }
                LinearRelationType::Ne => {
                    let zero =
                        D::single_value(ConcreteBitvector::zero(combination.constant.bound()));
                    let value = self.evaluate_combination(assignment, combination);
                    TypedEq::ne(value, zero)
                }
            };

            if system.universal {
                result = result.bit_and(relation_result);
            } else {
                result = result.bit_or(relation_result);
            }
        }
        result
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
{
    fn formula(bound: RBound, formula: FormulaId) -> Self;
}

impl EvaluableDomain for AbstractBitvector<RBound> {
    fn formula(bound: RBound, formula: FormulaId) -> Self {
        let _ = formula;
        Self::top(bound)
    }
}

impl EvaluableDomain for LinearBitvector {
    fn formula(bound: RBound, formula: FormulaId) -> Self {
        let mut coefficients = BTreeMap::new();
        coefficients.insert(formula, ConcreteBitvector::one(bound));
        LinearBitvector::Combination(LinearCombination {
            constant: ConcreteBitvector::zero(bound),
            coefficients,
        })
    }
}

impl<D: EvaluableDomain + Debug> Debug for Evaluator<'_, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut franz = f.debug_struct("Evaluator");

        struct FieldStr<'a>(&'a str);

        impl Debug for FieldStr<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        for (variable_id, width) in self.problem.variable_widths.iter().enumerate() {
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
