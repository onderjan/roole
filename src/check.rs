use crate::{
    domain::{
        bitvector::{RBound, abstr::AbstractBitvector},
        traits::forward::{Bitwise, HwArith, TypedEq},
    },
    formula::{BiOp, BiOperator, FormulaId, Operation, UniOp, UniOperator},
};

#[derive(Debug)]
pub struct Checker {
    pub variable_widths: Vec<u32>,
    pub operations: Vec<Operation>,
    pub assertion: FormulaId,
}

impl Checker {
    pub fn check(&self) {
        eprintln!("Should check-sat with {:#?}", self);

        let mut assignments = Vec::new();

        for width in self.variable_widths.iter().cloned() {
            assignments.push(AbstractBitvector::new(0, RBound::new(width))); //ThreeValued::new_unknown()
        }

        let result = self.eval_formula(&assignments, self.assertion);
        eprintln!("Formula evaluation result: {:?}", result);
    }

    pub fn eval_formula(
        &self,
        assignments: &[AbstractBitvector<RBound>],
        formula_id: FormulaId,
    ) -> AbstractBitvector<RBound> {
        let result = match formula_id {
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
                    }
                }
            },
        };

        eprintln!("Evaluated {:?} with result: {:?}", formula_id, result);
        result
    }
}
