use std::fmt::{Debug, UpperHex};

use crate::{
    domain::{
        bitvector::{
            RBound,
            abstr::{AbstractBitvector, BitvectorDomain},
        },
        traits::forward::{BExt, Bitwise, HwArith, HwShift, TypedCmp, TypedEq},
    },
    problem::formula::FormulaId,
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
    fn used_ids(&self) -> Option<Vec<FormulaId>>;
}

impl EvaluableDomain for AbstractBitvector<RBound> {
    fn used_ids(&self) -> Option<Vec<FormulaId>> {
        // no used ids tracked
        None
    }
}
