use crate::{
    domain::{
        bitvector::{RBound, abstr::AbstractBitvector},
        traits::forward::{Bitwise, HwArith, TypedEq},
    },
    formula::{BiOp, FormulaId, Operation, UniOp},
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
    ) -> (u32, AbstractBitvector<RBound>) {
        let result = match formula_id {
            FormulaId::Variable(variable_id) => (
                self.variable_widths[variable_id.0],
                assignments[variable_id.0],
            ),

            FormulaId::Operation(operation_id) => match &self.operations[operation_id.0] {
                Operation::UniOp(uni_op, inner) => {
                    let (width, inner) = self.eval_formula(assignments, *inner);
                    match uni_op {
                        UniOp::Not => (width, inner.bit_not()),
                    }
                }
                Operation::BiOp(bi_op, left, right) => {
                    let (left_width, left) = self.eval_formula(assignments, *left);
                    let (right_width, right) = self.eval_formula(assignments, *right);
                    assert_eq!(left_width, right_width);
                    let width = left_width;

                    let result = match bi_op {
                        BiOp::Add => left.add(right),
                        BiOp::Sub => left.sub(right),
                        BiOp::BitAnd => left.bit_and(right),
                        BiOp::BitOr => left.bit_or(right),
                        BiOp::BitXor => left.bit_xor(right),
                        BiOp::Eq => {
                            return (1, TypedEq::eq(left, right));
                        }
                    };
                    (width, result)
                }
            },
        };

        eprintln!("Evaluated {:?} with result: {:?}", formula_id, result);
        result
    }
}
