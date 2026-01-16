use std::fmt::Debug;

/// Formula id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormulaId {
    Variable(VariableId),
    Operation(OperationId),
}

/// Variable id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableId(pub usize);

/// Operation id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Clone, Debug)]
pub enum UniOperator {
    Not,
}

#[derive(Clone, Debug)]
pub enum BiOperator {
    Add,
    Sub,
    Mul,

    BitAnd,
    BitOr,
    BitXor,

    Eq,

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
        }
    }
}
