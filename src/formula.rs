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
    UniOp(UniOp, FormulaId),
    BiOp(BiOp, FormulaId, FormulaId),
}

#[derive(Clone, Debug)]
pub enum UniOp {
    Not,
}

#[derive(Clone, Debug)]
pub enum BiOp {
    Add,
    Sub,

    BitAnd,
    BitOr,
    BitXor,

    Eq,
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
                write!(f, "bv{}({})", width, value)
            }
            Operation::UniOp(op, inner) => {
                write!(f, "{:?}", op)?;
                let mut franz = f.debug_tuple("");
                franz.field(inner);
                franz.finish()
            }
            Operation::BiOp(op, left, right) => {
                write!(f, "{:?}", op)?;
                let mut franz = f.debug_tuple("");
                franz.field(left);
                franz.field(right);
                franz.finish()
            }
        }
    }
}
