use std::{fmt::Debug, num::NonZeroU32};

use bimap::BiBTreeMap;
use serde::{Deserialize, Serialize};

use crate::domain::bitvector::{
    BitvectorBound,
    abstr::linear::{LinearCombination, LinearSystem},
};

/// Formula id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FormulaId {
    Variable(VariableId),
    Operation(OperationId),
}

/// Variable id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VariableId(pub usize);

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
    LinearCombination(LinearCombination),
    LinearSystem(LinearSystem),
}

#[derive(Clone)]
pub struct UniOp {
    pub op: UniOperator,
    pub input_width: u32,
    pub inner: FormulaId,
}

#[derive(Clone)]
pub struct BiOp {
    pub op: BiOperator,
    pub input_width: u32,
    pub left: FormulaId,
    pub right: FormulaId,
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

#[derive(Clone, Copy, Debug)]
pub enum BiOperator {
    Add,
    Sub,
    Mul,

    BitAnd,
    BitOr,
    BitXor,

    Eq,
    Ne,
    Implies,

    Ult,
    Ule,
    Ugt,
    Uge,

    Slt,
    Sle,
    Sgt,
    Sge,

    Shl,
    Lshr,
    Ashr,
}

impl Operation {
    pub fn result_width(&self) -> u32 {
        match self {
            Operation::Constant(_value, width) => *width,
            Operation::UniOp(uni_op) => match uni_op.op {
                UniOperator::Not => uni_op.input_width,
            },
            Operation::BiOp(bi_op) => match bi_op.op {
                BiOperator::Eq
                | BiOperator::Ne
                | BiOperator::Ult
                | BiOperator::Ule
                | BiOperator::Ugt
                | BiOperator::Uge
                | BiOperator::Slt
                | BiOperator::Sle
                | BiOperator::Sgt
                | BiOperator::Sge => 1,
                _ => bi_op.input_width,
            },
            Operation::ExtOp(ext_op) => ext_op.output_width,
            Operation::IteOp(ite_op) => ite_op.width,
            Operation::ConcatOp(concat_op) => concat_op.left_width + concat_op.right_width,
            Operation::ExtractOp(extract_op) => extract_op.width.get(),
            Operation::LinearCombination(combination) => combination.constant.bound().width(),
            Operation::LinearSystem(_) => 1,
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
            Operation::LinearCombination(combination) => {
                combination.coefficients.keys().copied().collect()
            }
            Operation::LinearSystem(system) => system
                .relations
                .iter()
                .flat_map(|relation| relation.combination.coefficients.keys().copied())
                .collect(),
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
            Operation::BiOp(bi_op) => Operation::BiOp(BiOp {
                op: bi_op.op,
                input_width: bi_op.input_width,
                left: remap(bi_op.left),
                right: remap(bi_op.right),
            }),
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
            Operation::LinearCombination(combination) => {
                Operation::LinearCombination(combination.clone().remap(old_to_new))
            }
            Operation::LinearSystem(system) => {
                eprintln!("Remapping system: {:?} with {:?}", system, old_to_new);
                let mut relations = system.relations.clone();
                for relation in &mut relations {
                    relation.combination = relation.combination.clone().remap(old_to_new);
                }
                let result = Operation::LinearSystem(LinearSystem {
                    universal: system.universal,
                    relations,
                });

                eprintln!("Remapped: {:?}", result);
                result
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

impl Debug for VariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
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
            Operation::BiOp(BiOp {
                op,
                input_width: width,
                left,
                right,
            }) => {
                write!(f, "{:?}_{}({:?},{:?})", op, width, left, right)
            }
            Operation::ExtOp(ExtOp {
                signed,
                input_width,
                output_width,
                inner,
            }) => {
                write!(
                    f,
                    "{:?}_{}_{}({:?})",
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
            Operation::LinearCombination(combination) => {
                write!(f, "linear_combination({:?})", combination)
            }
            Operation::LinearSystem(system) => {
                write!(f, "linear_system({:?})", system)
            }
        }
    }
}
