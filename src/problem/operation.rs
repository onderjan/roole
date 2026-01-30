use std::{fmt::Debug, num::NonZeroU32};

use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::BExt,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

mod bi;
mod linear;

pub use bi::{BiOp, BiOperator};

pub use linear::{LinearCombination, LinearOperation, LinearSystem};

/// Operation id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OperationId(pub usize);

/// Operation on bitvector(s).
///
/// The operations store ids of their inputs,
/// so they can be stored in vectors instead of
/// being represented in memory as a tree / directed graph.
#[derive(Clone)]
pub enum Operation {
    Constant(u64, u32),
    UniOp(UniOp),
    BiOp(BiOp),
    ExtOp(ExtOp),
    IteOp(IteOp),
    ConcatOp(ConcatOp),
    ExtractOp(ExtractOp),
    Linear(LinearOperation),
}

#[derive(Clone)]
pub struct UniOp {
    pub op: UniOperator,
    pub input_width: u32,
    pub inner: FormulaId,
}

#[derive(Clone)]
pub struct ExtOp {
    pub signed: bool,
    pub input_width: u32,
    pub output_width: u32,
    pub inner: FormulaId,
}

#[derive(Clone)]
pub struct IteOp {
    pub condition: FormulaId,
    pub width: u32,
    pub formula_then: FormulaId,
    pub formula_else: FormulaId,
}

#[derive(Clone)]
pub struct ConcatOp {
    pub left_width: u32,
    pub left: FormulaId,
    pub right_width: u32,
    pub right: FormulaId,
}

#[derive(Clone)]
pub struct ExtractOp {
    pub inner: FormulaId,
    pub lsb: u32,
    pub width: NonZeroU32,
}

#[derive(Clone, Copy, Debug)]
pub enum UniOperator {
    Not,
}

impl Operation {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match self {
            Operation::Constant(value, width) => {
                let concrete = ConcreteBitvector::new(*value, RBound::new(*width));
                D::single_value(concrete)
            }
            Operation::UniOp(UniOp {
                op,
                input_width: _,
                inner,
            }) => {
                let inner = (fetch)(*inner);
                match op {
                    UniOperator::Not => inner.bit_not(),
                }
            }
            Operation::BiOp(bi_op) => bi_op.evaluate(fetch),
            Operation::ExtOp(ExtOp {
                signed,
                input_width: _,
                output_width,
                inner,
            }) => {
                let inner = (fetch)(*inner);
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
                let condition = (fetch)(*condition);
                assert_eq!(condition.bound().width(), 1);

                if let Some(condition_value) = condition.concrete_value() {
                    if condition_value.is_nonzero() {
                        // only then taken
                        (fetch)(*formula_then)
                    } else {
                        // only else taken
                        (fetch)(*formula_else)
                    }
                } else {
                    // both can be taken, join them
                    let value_then = (fetch)(*formula_then);
                    let value_else = (fetch)(*formula_else);
                    value_then.join(&value_else)
                }
            }
            Operation::ConcatOp(concat_op) => {
                let left = (fetch)(concat_op.left);
                let right = (fetch)(concat_op.right);

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
                let inner = (fetch)(extract_op.inner);

                assert!(inner.bound().width() >= extract_op.lsb + extract_op.width.get());

                // shift right by lsb
                // it should not matter which shift it is, perform it unsigned
                let concrete_rhs = ConcreteBitvector::new(extract_op.lsb.into(), inner.bound());
                let inner = inner.logic_shr(D::single_value(concrete_rhs));

                // narrow to extraction width
                inner.uext(RBound::new(extract_op.width.get()))
            }
            Operation::Linear(linear) => linear.evaluate(fetch),
        }
    }

    pub fn result_width(&self) -> u32 {
        match self {
            Operation::Constant(_value, width) => *width,
            Operation::UniOp(uni_op) => match uni_op.op {
                UniOperator::Not => uni_op.input_width,
            },
            Operation::BiOp(bi_op) => bi_op.result_width(),
            Operation::ExtOp(ext_op) => ext_op.output_width,
            Operation::IteOp(ite_op) => ite_op.width,
            Operation::ConcatOp(concat_op) => concat_op.left_width + concat_op.right_width,
            Operation::ExtractOp(extract_op) => extract_op.width.get(),
            Operation::Linear(linear) => linear.result_bound().width(),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match self {
            Operation::Constant(_, _) => vec![],
            Operation::UniOp(uni_op) => vec![uni_op.inner],
            Operation::BiOp(bi_op) => vec![bi_op.left, bi_op.right],
            Operation::ExtOp(ext_op) => vec![ext_op.inner],
            Operation::IteOp(ite_op) => {
                vec![ite_op.condition, ite_op.formula_then, ite_op.formula_else]
            }
            Operation::ConcatOp(concat_op) => vec![concat_op.left, concat_op.right],
            Operation::ExtractOp(extract_op) => vec![extract_op.inner],
            Operation::Linear(linear) => linear.used_ids(),
        }
    }

    pub fn remapped(&self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) -> Self {
        let remap = |formula_id| {
            let Some(new_id) = old_to_new.get_by_left(&formula_id) else {
                panic!("Used formula id {:?} should be remappable", formula_id);
            };
            *new_id
        };

        match self {
            Operation::Constant(_, _) => self.clone(),
            Operation::UniOp(uni_op) => Operation::UniOp(UniOp {
                op: uni_op.op,
                input_width: uni_op.input_width,
                inner: remap(uni_op.inner),
            }),
            Operation::BiOp(bi_op) => Operation::BiOp(bi_op.remapped(old_to_new)),
            Operation::ExtOp(ext_op) => Operation::ExtOp(ExtOp {
                signed: ext_op.signed,
                input_width: ext_op.input_width,
                output_width: ext_op.output_width,
                inner: remap(ext_op.inner),
            }),
            Operation::IteOp(ite_op) => Operation::IteOp(IteOp {
                condition: remap(ite_op.condition),
                width: ite_op.width,
                formula_then: remap(ite_op.formula_then),
                formula_else: remap(ite_op.formula_else),
            }),
            Operation::ConcatOp(concat_op) => Operation::ConcatOp(ConcatOp {
                left_width: concat_op.left_width,
                left: remap(concat_op.left),
                right_width: concat_op.right_width,
                right: remap(concat_op.right),
            }),
            Operation::ExtractOp(extract_op) => Operation::ExtractOp(ExtractOp {
                inner: remap(extract_op.inner),
                lsb: extract_op.lsb,
                width: extract_op.width,
            }),
            Operation::Linear(linear) => {
                // TODO: rewrite remapped to use mutable reference
                let mut linear = linear.clone();
                linear.remap(old_to_new);
                Operation::Linear(linear)
            }
        }
    }
}

impl Debug for FormulaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormulaId::Variable(variable_id) => variable_id.fmt(f),
            FormulaId::Operation(operation_id) => operation_id.fmt(f),
        }
    }
}

impl Debug for OperationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

impl Debug for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Constant(value, width) => {
                write!(f, "bv_{}({})", width, value)
            }
            Operation::UniOp(UniOp {
                op,
                input_width: width,
                inner,
            }) => {
                write!(f, "{:?}_{}({:?})", op, width, inner)
            }
            Operation::BiOp(bi_op) => Debug::fmt(&bi_op, f),
            Operation::ExtOp(ExtOp {
                signed,
                input_width,
                output_width,
                inner,
            }) => {
                write!(
                    f,
                    "{}_{}_{}({:?})",
                    if *signed { "sext" } else { "uext" },
                    input_width,
                    output_width,
                    inner
                )
            }
            Operation::IteOp(IteOp {
                condition,
                width,
                formula_then,
                formula_else,
            }) => {
                write!(
                    f,
                    "ite_{}({:?},{:?},{:?})",
                    width, condition, formula_then, formula_else
                )
            }
            Operation::ConcatOp(ConcatOp {
                left_width,
                left,
                right_width,
                right,
            }) => write!(
                f,
                "concat_{}_{}({:?},{:?})",
                left_width, right_width, left, right
            ),
            Operation::ExtractOp(ExtractOp { inner, lsb, width }) => {
                write!(f, "extract_{}_{}({:?})", lsb + width.get() - 1, lsb, inner)
            }
            Operation::Linear(linear) => Debug::fmt(&linear, f),
        }
    }
}
