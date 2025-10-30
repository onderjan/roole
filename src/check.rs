use num::{BigUint, One, ToPrimitive, Zero};

use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{AbstractBitvector, BitvectorDomain, three_valued::ThreeValuedBitvector},
            compute_u64_mask,
            concr::ConcreteBitvector,
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

struct SearchSpaceInfo {
    num_leafs: BigUint,
    num_nodes: BigUint,
    opened_nodes: BigUint,
}

impl Checker {
    pub fn check(&self) {
        eprintln!("Should check-sat with {:#?}", self);

        //self.brute_force();
        self.recursive_dpll();
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

    pub fn recursive_dpll(&self) {
        let mut total_width = 0u128;
        let mut assignments = Vec::new();
        for width in self.variable_widths.iter().cloned() {
            assignments.push(AbstractBitvector::new_unknown(RBound::new(width)));
            total_width = total_width
                .checked_add(width as u128)
                .expect("Total width should be in u128");
        }

        let num_leafs = BigUint::one() << total_width;
        let num_nodes = (num_leafs.clone() * 2u32) - 1u32;

        let mut info = SearchSpaceInfo {
            num_leafs,
            num_nodes,
            opened_nodes: BigUint::zero(),
        };

        if !self.dpll_recursion(&mut info, &mut assignments, 0, 0) {
            eprintln!("Unsatisfiable");
        }

        let precision_const = 1_000_000u32;

        let percent_opened: f64 = (info.opened_nodes.clone() * precision_const
            / info.num_nodes.clone())
        .to_f64()
        .unwrap_or(f64::NAN)
            / (precision_const as f64)
            * 100.;

        eprintln!(
            "Info: {} leafs, {} nodes, {} opened ({:.2}%)",
            info.num_leafs, info.num_nodes, info.opened_nodes, percent_opened
        );
    }

    fn dpll_recursion(
        &self,
        info: &mut SearchSpaceInfo,
        assignments: &mut [AbstractBitvector<RBound>],
        variable_index: usize,
        bit_index: u32,
    ) -> bool {
        info.opened_nodes += 1u32;

        let result = self.eval_formula(assignments, self.assertion);

        if let Some(concrete_result) = result.concrete_value() {
            if concrete_result.is_nonzero() {
                eprintln!("Satisfiable: {:?}", assignments);
                return true;
            } else {
                // unsatisfiable branch
                return false;
            }
        };

        let original_value = assignments[variable_index];
        let bound = original_value.bound();
        let bit_index_mask = ConcreteBitvector::from_masked_u64(1 << bit_index, bound);

        let mut next_variable_index = variable_index;
        let mut next_bit_index = bit_index + 1;
        if next_bit_index >= bound.width() {
            next_bit_index = 0;
            next_variable_index += 1;
        }

        // assign zero

        assignments[variable_index] = original_value.bit_and(
            ThreeValuedBitvector::from_concrete_value(bit_index_mask.bit_not()),
        );

        if self.dpll_recursion(info, assignments, next_variable_index, next_bit_index) {
            return true;
        }

        // assign one

        assignments[variable_index] =
            original_value.bit_or(ThreeValuedBitvector::from_concrete_value(bit_index_mask));

        if self.dpll_recursion(info, assignments, next_variable_index, next_bit_index) {
            return true;
        }

        // go back to unknown
        assignments[variable_index] = original_value;

        false
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
