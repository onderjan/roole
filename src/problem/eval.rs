use super::formula::{BiOp, BiOperator, ExtOp, FormulaId, IteOp, Operation, UniOp, UniOperator};
use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
        },
        traits::{
            Join,
            forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
        },
    },
    problem::{Problem, assignment::Assignment},
};

/// Evaluates this problem assertion on the given variable assignment.
///
/// The assignment structure must correspond to the problem variables.
pub fn evaluate(problem: &Problem, assignment: &Assignment) -> AbstractBitvector<RBound> {
    let evaluator = Evaluator { problem };

    evaluator.eval_formula(assignment, problem.assertion)
}

struct Evaluator<'a> {
    problem: &'a Problem,
}

impl Evaluator<'_> {
    /// Evaluates a formula of this problem with the given assignment.
    fn eval_formula(
        &self,
        assignment: &Assignment,
        formula_id: FormulaId,
    ) -> AbstractBitvector<RBound> {
        match formula_id {
            FormulaId::Variable(variable_id) => assignment.values[variable_id.0],

            FormulaId::Operation(operation_id) => {
                self.eval_operation(assignment, &self.problem.operations[operation_id.0])
            }
        }
    }

    /// Evaluates an operation of this problem with the given assignment.
    fn eval_operation(
        &self,
        assignment: &Assignment,
        operation: &Operation,
    ) -> AbstractBitvector<RBound> {
        match operation {
            Operation::Constant(value, width) => {
                AbstractBitvector::new(*value, RBound::new(*width))
            }
            Operation::UniOp(UniOp {
                op,
                input_width: _,
                inner,
            }) => {
                let inner = self.eval_formula(assignment, *inner);
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
                let left = self.eval_formula(assignment, *left);
                let right = self.eval_formula(assignment, *right);

                match op {
                    BiOperator::Add => left.add(right),
                    BiOperator::Sub => left.sub(right),
                    BiOperator::Mul => left.mul(right),

                    BiOperator::BitAnd => left.bit_and(right),
                    BiOperator::BitOr => left.bit_or(right),
                    BiOperator::BitXor => left.bit_xor(right),

                    BiOperator::Eq => TypedEq::eq(left, right),
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
                let inner = self.eval_formula(assignment, *inner);
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
                let condition = self.eval_formula(assignment, *condition);
                assert_eq!(condition.bound().width(), 1);

                if let Some(condition_value) = condition.concrete_value() {
                    if condition_value.is_nonzero() {
                        // only then taken
                        self.eval_formula(assignment, *formula_then)
                    } else {
                        // only else taken
                        self.eval_formula(assignment, *formula_else)
                    }
                } else {
                    // both can be taken, join them
                    let value_then = self.eval_formula(assignment, *formula_then);
                    let value_else = self.eval_formula(assignment, *formula_else);
                    value_then.join(&value_else)
                }
            }
            Operation::ConcatOp(concat_op) => {
                let left = self.eval_formula(assignment, concat_op.left);
                let right = self.eval_formula(assignment, concat_op.right);

                assert_eq!(left.bound().width(), concat_op.left_width);
                assert_eq!(right.bound().width(), concat_op.right_width);

                let result_width = concat_op.left_width + concat_op.right_width;
                let result_bound = RBound::new(result_width);

                // zero-extend both to result width
                let left = left.uext(result_bound);
                let right = right.uext(result_bound);

                // shift left by right width
                let right_width_bitvector =
                    AbstractBitvector::new(concat_op.right_width as u64, result_bound);
                let left = left.logic_shl(right_width_bitvector);

                // bit-or both
                left.bit_or(right)
            }
            Operation::ExtractOp(extract_op) => {
                let inner = self.eval_formula(assignment, extract_op.inner);

                assert!(inner.bound().width() >= extract_op.lsb + extract_op.width.get());

                // shift right by lsb
                // it should not matter which shift it is, perform it unsigned
                let inner =
                    inner.logic_shr(AbstractBitvector::new(extract_op.lsb.into(), inner.bound()));

                // narrow to extraction width
                inner.uext(RBound::new(extract_op.width.get()))
            }
        }
    }
}
