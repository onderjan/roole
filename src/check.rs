use crate::{
    bitvector::{abstr::ThreeValued, concr::RUnsignedU64},
    formula::Formula,
};

#[derive(Debug)]
pub struct Checker {
    pub variable_widths: Vec<u32>,
    pub assertion: Formula,
}

impl Checker {
    pub fn check(&self) {
        eprintln!("Should check-sat with {:#?}", self);

        let mut assignments = Vec::new();

        for width in self.variable_widths.iter().cloned() {
            assignments.push(ThreeValued::new(RUnsignedU64(0), width)); //ThreeValued::new_unknown()
        }

        let result = self.eval_formula(&assignments, &self.assertion);
        eprintln!("Formula evaluation result: {:?}", result);
    }

    pub fn eval_formula(
        &self,
        assignments: &[ThreeValued<RUnsignedU64>],
        formula: &Formula,
    ) -> (u32, ThreeValued<RUnsignedU64>) {
        let result = match formula {
            Formula::Variable(variable_id) => (
                self.variable_widths[variable_id.0],
                assignments[variable_id.0],
            ),
            Formula::UniOp(uni_op, inner) => {
                let (width, inner) = self.eval_formula(assignments, inner);
                match uni_op {
                    crate::formula::UniOp::Not => (width, inner.not(width)),
                }
            }
            Formula::BiOp(bi_op, left, right) => {
                let (left_width, left) = self.eval_formula(assignments, left);
                let (right_width, right) = self.eval_formula(assignments, right);
                assert_eq!(left_width, right_width);
                let width = left_width;

                let result = match bi_op {
                    crate::formula::BiOp::Add => left.add(right, width),
                    crate::formula::BiOp::Sub => left.sub(right, width),
                    crate::formula::BiOp::BitAnd => left.bitand(right, width),
                    crate::formula::BiOp::BitOr => left.bitor(right, width),
                    crate::formula::BiOp::BitXor => left.bitxor(right, width),
                    crate::formula::BiOp::Eq => {
                        return (1, left.eq(right, width));
                    }
                };
                (width, result)
            }
        };

        eprintln!("Evaluated {:?} with result: {:?}", formula, result);
        result
    }
}
