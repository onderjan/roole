use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::traits::forward::{TypedCmp, TypedEq},
    problem::{evaluator::EvaluableDomain, formula::FormulaId},
};

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

    Udiv,
    Urem,
    Sdiv,
    Srem,

    BitAnd,
    BitOr,
    BitXor,
    BitNand,
    BitNor,
    BitXnor,

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
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let left = (fetch)(self.left);
        let right = (fetch)(self.right);

        match self.op {
            BiOperator::Add => left.add(right),
            BiOperator::Sub => left.sub(right),
            BiOperator::Mul => left.mul(right),
            BiOperator::Udiv => left.udiv_wrapping_or_all_ones(right),
            BiOperator::Urem => left.urem_wrapping_or_dividend(right),
            BiOperator::Sdiv => left.sdiv_wrapping_by_quadrants(right),
            BiOperator::Srem => left.srem_wrapping_by_quadrants(right),

            BiOperator::BitAnd => left.bit_and(right),
            BiOperator::BitOr => left.bit_or(right),
            BiOperator::BitXor => left.bit_xor(right),
            BiOperator::BitNand => left.bit_and(right).bit_not(),
            BiOperator::BitNor => left.bit_or(right).bit_not(),
            BiOperator::BitXnor => left.bit_xor(right).bit_not(),

            BiOperator::Eq => TypedEq::eq(left, right),
            BiOperator::Ne => TypedEq::ne(left, right),
            BiOperator::Implies => (left.bit_not()).bit_or(right),

            BiOperator::Ult => TypedCmp::ult(left, right),
            BiOperator::Ule => TypedCmp::ule(left, right),
            BiOperator::Ugt => TypedCmp::ule(left, right).bit_not(),
            BiOperator::Uge => TypedCmp::ult(left, right).bit_not(),
            BiOperator::Slt => TypedCmp::slt(left, right),
            BiOperator::Sle => TypedCmp::sle(left, right),
            BiOperator::Sgt => TypedCmp::sle(left, right).bit_not(),
            BiOperator::Sge => TypedCmp::slt(left, right).bit_not(),

            BiOperator::Shl => left.logic_shl(right),
            BiOperator::Lshr => left.logic_shr(right),
            BiOperator::Ashr => left.arith_shr(right),
        }
    }

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

    pub(super) fn remapped(&self, old_to_new: &BTreeMap<FormulaId, FormulaId>) -> Self {
        let remap = |formula_id| {
            let Some(new_id) = old_to_new.get(&formula_id) else {
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
