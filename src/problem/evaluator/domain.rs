use std::fmt::{Debug, UpperHex};

use crate::{
    domain::{
        bitvector::{
            RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
        },
        traits::forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    },
    problem::{formula::FormulaId, symbolic::SymbolicDomain},
};

pub trait EvaluableDomain:
    BitvectorDomain<Bound = RBound>
    + HwArith
    + Bitwise
    + TypedEq<Output = Self>
    + TypedCmp<Output = Self>
    + HwShift<Output = Self>
    + BExt<RBound, Output = Self>
    + Debug
    + UpperHex
{
    fn formula(formula: FormulaId, bound: RBound) -> Self;
    fn used_ids(&self) -> Vec<FormulaId>;
}

impl EvaluableDomain for AbstractBitvector<RBound> {
    fn formula(formula: FormulaId, bound: RBound) -> Self {
        let _ = formula;
        Self::top(bound)
    }
    fn used_ids(&self) -> Vec<FormulaId> {
        // no used ids
        Vec::new()
    }
}

impl EvaluableDomain for SymbolicDomain {
    fn formula(formula_id: FormulaId, bound: RBound) -> Self {
        SymbolicDomain::from_formula(formula_id, bound)
    }

    fn used_ids(&self) -> Vec<FormulaId> {
        SymbolicDomain::used_ids(self)
    }
}
