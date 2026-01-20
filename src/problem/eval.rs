use super::formula::{BiOp, BiOperator, ExtOp, FormulaId, IteOp, Operation, UniOp, UniOperator};
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    },
    problem::{Problem, assignment::Assignment},
};

pub trait EvaluableDomain:
    BitvectorDomain<Bound = RBound>
    + HwArith
    + Bitwise
    + TypedEq<Output = Self>
    + TypedCmp<Output = Self>
    + HwShift<Output = Self>
    + BExt<RBound, Output = Self>
{
}

impl<
    T: BitvectorDomain<Bound = RBound>
        + HwArith
        + Bitwise
        + TypedEq<Output = Self>
        + TypedCmp<Output = Self>
        + HwShift<Output = Self>
        + BExt<RBound, Output = Self>,
> EvaluableDomain for T
{
}

#[derive(Debug)]
pub struct Evaluator<'a, D: EvaluableDomain> {
    problem: &'a Problem,
    // the results are indexed by FormulaId
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
                        self.results[operation_id.0] =
                            Some(self.evaluate_operation(assignment, operation))
                    } else {
                        let dependencies = self.dependencies(operation);

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

    fn dependencies(&self, operation: &Operation) -> Vec<FormulaId> {
        match operation {
            Operation::Constant(_, _) => {
                vec![]
            }
            Operation::UniOp(uni_op) => {
                vec![uni_op.inner]
            }
            Operation::BiOp(bi_op) => {
                vec![bi_op.left, bi_op.right]
            }
            Operation::ExtOp(ext_op) => {
                vec![ext_op.inner]
            }
            Operation::IteOp(ite_op) => {
                vec![ite_op.condition, ite_op.formula_then, ite_op.formula_else]
            }
            Operation::ConcatOp(concat_op) => {
                vec![concat_op.left, concat_op.right]
            }
            Operation::ExtractOp(extract_op) => {
                vec![extract_op.inner]
            }
        }
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
        }
    }
}
