use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
            compute_u64_mask,
        },
        traits::{
            Join,
            forward::{BExt, Bitwise, HwArith, HwShift, TypedEq},
        },
    },
    formula::{BiOp, BiOperator, ExtOp, FormulaId, IteOp, Operation, UniOp, UniOperator},
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

        self.brute_force();
    }

    pub fn brute_force(&self) {
        let mut assignments = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            assignments.push(AbstractBitvector::new(0, RBound::new(width)));
        }

        let mut iterators: Vec<_> = self
            .variable_widths
            .iter()
            .map(|width| (width, 0..compute_u64_mask(*width)))
            .collect();

        let mut satisfiable = false;

        loop {
            let mut early = false;
            for (index, (width, iterator)) in iterators.iter_mut().enumerate() {
                match iterator.next() {
                    Some(value) => {
                        assignments[index] = AbstractBitvector::new(value, RBound::new(**width));

                        early = true;
                        break;
                    }
                    None => {
                        *iterator = 0..compute_u64_mask(**width);
                        assignments[index] = AbstractBitvector::new(0, RBound::new(**width));
                    }
                }
            }
            if !early {
                break;
            }

            let result = self.eval_formula(&assignments, self.assertion);

            let Some(concrete_result) = result.concrete_value() else {
                panic!("Concrete values should produce concrete result");
            };

            //eprintln!("Assignments: {:?}, result: {:?}", assignments, result);

            if concrete_result.is_nonzero() {
                satisfiable = true;
                eprintln!("Satisfiable: {:?}", assignments);
                break;
            }
        }
        if !satisfiable {
            eprintln!("Unsatisfiable");
        }
    }

    pub fn eval_formula(
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
