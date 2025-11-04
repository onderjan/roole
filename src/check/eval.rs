use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
        },
        traits::{
            Join,
            forward::{BExt, Bitwise, HwArith, HwShift, TypedEq},
        },
    },
    formula::{BiOp, BiOperator, ExtOp, FormulaId, IteOp, Operation, UniOp, UniOperator},
};

impl super::Checker {
    pub(super) fn eval_formula(
        &self,
        assignments: &[AbstractBitvector<RBound>],
        formula_id: FormulaId,
    ) -> AbstractBitvector<RBound> {
        //eprintln!("Evaluated {:?} with result: {:?}", formula_id, result);
        match formula_id {
            FormulaId::Variable(variable_id) => assignments[variable_id.0],

            FormulaId::Operation(operation_id) => match &self.operations[operation_id.0] {
                Operation::Constant(value, width) => {
                    AbstractBitvector::new(*value, RBound::new(*width))
                }
                Operation::UniOp(UniOp {
                    op,
                    input_width: _,
                    inner,
                }) => {
                    let inner = self.eval_formula(assignments, *inner);
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
                    let left = self.eval_formula(assignments, *left);
                    let right = self.eval_formula(assignments, *right);

                    match op {
                        BiOperator::Add => left.add(right),
                        BiOperator::Sub => left.sub(right),
                        BiOperator::BitAnd => left.bit_and(right),
                        BiOperator::BitOr => left.bit_or(right),
                        BiOperator::BitXor => left.bit_xor(right),
                        BiOperator::Eq => TypedEq::eq(left, right),
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
                    let inner = self.eval_formula(assignments, *inner);
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
                    let condition = self.eval_formula(assignments, *condition);
                    assert_eq!(condition.bound().width(), 1);

                    if let Some(condition_value) = condition.concrete_value() {
                        if condition_value.is_nonzero() {
                            // only then taken
                            self.eval_formula(assignments, *formula_then)
                        } else {
                            // only else taken
                            self.eval_formula(assignments, *formula_else)
                        }
                    } else {
                        // both can be taken, join them
                        let value_then = self.eval_formula(assignments, *formula_then);
                        let value_else = self.eval_formula(assignments, *formula_else);
                        value_then.join(&value_else)
                    }
                }
            },
        }
    }
}
