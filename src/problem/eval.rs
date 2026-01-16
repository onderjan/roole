use rapidhash::RapidHashMap;

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

pub struct Evaluator<'a> {
    problem: &'a Problem,
    results: RapidHashMap<FormulaId, AbstractBitvector<RBound>>,
}

impl<'a> Evaluator<'a> {
    pub fn new(problem: &'a Problem) -> Self {
        Self {
            problem,
            results: RapidHashMap::default(),
        }
    }

    pub fn problem(&self) -> &'a Problem {
        self.problem
    }

    /// Evaluates this problem assertion on the given variable assignment.
    ///
    /// The assignment structure must correspond to the problem variables.
    pub fn evaluate(&mut self, assignment: &Assignment) -> AbstractBitvector<RBound> {
        // must clear previous results to work with new assignment
        // keep the allocated memory for reuse
        self.results.clear();

        let mut op_stack = vec![(self.problem.assertion, false)];

        while let Some((formula_id, evaluated)) = op_stack.pop() {
            let result = match formula_id {
                FormulaId::Variable(variable_id) => Some(assignment.values[variable_id.0]),
                FormulaId::Operation(operation_id) => {
                    if evaluated {
                        Some(self.evaluate_operation(&self.problem.operations[operation_id.0]))
                    } else {
                        let dependencies =
                            self.dependencies(&self.problem.operations[operation_id.0]);

                        op_stack.push((formula_id, true));
                        for dependency in dependencies.into_iter().rev() {
                            op_stack.push((dependency, false));
                        }

                        None
                    }
                }
            };

            if let Some(result) = result {
                self.results.insert(formula_id, result);
            }
        }

        self.results
            .remove(&self.problem.assertion)
            .expect("Problem assertion should have a result after evaluation")
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

    fn fetch_result(&self, formula_id: FormulaId) -> AbstractBitvector<RBound> {
        let Some(result) = self.results.get(&formula_id) else {
            panic!(
                "Fetched result of formula {:?} should be already computed",
                formula_id
            );
        };
        *result
    }

    fn evaluate_operation(&self, operation: &Operation) -> AbstractBitvector<RBound> {
        match operation {
            Operation::Constant(value, width) => {
                AbstractBitvector::new(*value, RBound::new(*width))
            }
            Operation::UniOp(UniOp {
                op,
                input_width: _,
                inner,
            }) => {
                let inner = self.fetch_result(*inner);
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
                let left = self.fetch_result(*left);
                let right = self.fetch_result(*right);

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
                let inner = self.fetch_result(*inner);
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
                let condition = self.fetch_result(*condition);
                assert_eq!(condition.bound().width(), 1);

                if let Some(condition_value) = condition.concrete_value() {
                    if condition_value.is_nonzero() {
                        // only then taken
                        self.fetch_result(*formula_then)
                    } else {
                        // only else taken
                        self.fetch_result(*formula_else)
                    }
                } else {
                    // both can be taken, join them
                    let value_then = self.fetch_result(*formula_then);
                    let value_else = self.fetch_result(*formula_else);
                    value_then.join(&value_else)
                }
            }
            Operation::ConcatOp(concat_op) => {
                let left = self.fetch_result(concat_op.left);
                let right = self.fetch_result(concat_op.right);

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
                let inner = self.fetch_result(extract_op.inner);

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
