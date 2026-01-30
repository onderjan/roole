use std::fmt::Debug;

use bimap::BiBTreeMap;

use crate::problem::formula::FormulaId;

#[derive(Clone)]
pub struct BiOp {
    pub op: BiOperator,
    pub input_width: u32,
    pub left: FormulaId,
    pub right: FormulaId,
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

impl BiOp {
    pub(super) fn result_width(&self) -> u32 {
        match self.op {
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
            _ => self.input_width,
        }
    }

    pub(super) fn remapped(&self, old_to_new: &BiBTreeMap<FormulaId, FormulaId>) -> Self {
        let remap = |formula_id| {
            let Some(new_id) = old_to_new.get_by_left(&formula_id) else {
                panic!("Used formula id {:?} should be remappable", formula_id);
            };
            *new_id
        };

        BiOp {
            op: self.op,
            input_width: self.input_width,
            left: remap(self.left),
            right: remap(self.right),
        }
    }
}

impl Debug for BiOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}_{}({:?},{:?})",
            self.op, self.input_width, self.left, self.right
        )
    }
}
