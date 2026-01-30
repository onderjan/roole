use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::problem::operation::OperationId;

/// Formula id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FormulaId {
    Variable(VariableId),
    Operation(OperationId),
}

/// Variable id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VariableId(pub usize);

/// Bitvector variable.
#[derive(Clone, Debug)]
pub struct Variable {
    pub width: u32,
}

impl Debug for VariableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}
