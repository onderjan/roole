use serde::{Deserialize, Serialize};

mod format;
pub mod operation;

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

/// Bitvector variable.
#[derive(Clone)]
pub struct Variable {
    pub width: u32,
}

impl FormulaId {
    pub fn operation_id(self) -> Option<OperationId> {
        if let Self::Operation(operation_id) = self {
            Some(operation_id)
        } else {
            None
        }
    }
}

impl OperationId {
    pub fn formula_id(self) -> FormulaId {
        FormulaId::Operation(self)
    }
}
