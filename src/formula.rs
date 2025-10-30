use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FormulaId {
    Variable(VariableId),
    Operation(OperationId),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VariableId(pub usize);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(pub usize);

#[derive(Clone)]
pub enum Operation {
    Constant(u64, u32),
    UniOp(UniOp),
    BiOp(BiOp),
    ExtOp(ExtOp),
    IteOp(IteOp),
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

#[derive(Clone, Debug)]
pub enum UniOperator {
    Not,
}

#[derive(Clone, Debug)]
pub enum BiOperator {
    Add,
    Sub,

    BitAnd,
    BitOr,
    BitXor,

    Eq,

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
                BiOperator::Eq => 1,
                _ => bi_op.input_width,
            },
            Operation::ExtOp(ext_op) => ext_op.output_width,
            Operation::IteOp(ite_op) => ite_op.width,
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
                write!(f, "{:?}_{}", op, width)?;
                let mut franz = f.debug_tuple("");
                franz.field(inner);
                franz.finish()
            }
            Operation::BiOp(BiOp {
                op,
                input_width: width,
                left,
                right,
            }) => {
                write!(f, "{:?}_{}", op, width)?;
                let mut franz = f.debug_tuple("");
                franz.field(left);
                franz.field(right);
                franz.finish()
            }
            Operation::ExtOp(ExtOp {
                signed,
                input_width,
                output_width,
                inner,
            }) => {
                write!(
                    f,
                    "{:?}_{}_{}",
                    if *signed { "sext" } else { "uext" },
                    input_width,
                    output_width
                )?;
                let mut franz = f.debug_tuple("");
                franz.field(inner);
                franz.finish()
            }
            Operation::IteOp(IteOp {
                condition,
                width,
                formula_then,
                formula_else,
            }) => {
                write!(f, "ite_{}", width)?;
                let mut franz = f.debug_tuple("");
                franz.field(condition);
                franz.field(formula_then);
                franz.field(formula_else);
                franz.finish()
            }
        }
    }
}
