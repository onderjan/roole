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
    problem::assignment::Assignment,
};

impl super::Problem {
    /// Evaluates a formula of this problem with the given assignment.
    pub(super) fn eval_formula(
        &self,
        assignment: &Assignment,
        formula_id: FormulaId,
    ) -> AbstractBitvector<RBound> {
        match formula_id {
            FormulaId::Variable(variable_id) => assignment.values[variable_id.0],

            FormulaId::Operation(operation_id) => {
                self.eval_operation(assignment, &self.operations[operation_id.0])
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
        }
    }
}
