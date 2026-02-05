use serde::{Deserialize, Serialize};

use crate::domain::bitvector::BitvectorBound;
use crate::domain::bitvector::concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector};

use super::super::traits::Join;
use super::bound::{CBound, RBound};
use std::fmt::Display;
use std::hash::Hash;

/*
pub mod combined;
pub mod dual_interval;
pub mod eq_domain;
*/
pub mod three_valued;

pub trait BitvectorDomain: Clone + Hash + Join + PartialEq + Eq {
    type Bound: BitvectorBound;

    fn bound(&self) -> Self::Bound;

    fn single_value(value: ConcreteBitvector<Self::Bound>) -> Self;
    fn top(bound: Self::Bound) -> Self;

    fn concrete_value(&self) -> Option<ConcreteBitvector<Self::Bound>>;
}

pub trait ExtendedBitvectorDomain: BitvectorDomain {
    type General<X: BitvectorBound>: BitvectorDomain<Bound = X>;

    fn meet(self, other: &Self) -> Option<Self>;

    fn umin(&self) -> UnsignedBitvector<Self::Bound>;
    fn umax(&self) -> UnsignedBitvector<Self::Bound>;
    fn smin(&self) -> SignedBitvector<Self::Bound>;
    fn smax(&self) -> SignedBitvector<Self::Bound>;

    fn display(&self) -> BitvectorDisplay;

    fn get_tracker(&self) -> Option<u32> {
        None
    }
    fn assign_tracker(&mut self, _tracker: Option<u32>) {}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DomainDisplay {
    Value(String),
    Tracker(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitvectorDisplay {
    domains: Vec<DomainDisplay>,
}

pub type AbstractBitvector<B> = three_valued::ThreeValuedBitvector<B>;

pub type RBitvector = AbstractBitvector<RBound>;
pub type CBitvector<const W: u32> = AbstractBitvector<CBound<W>>;

pub type PanicBitvector = AbstractBitvector<CBound<32>>;

impl Display for BitvectorDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.domains.is_empty() {
            return write!(f, "⊤");
        }

        let mut first = true;

        for domain in &self.domains {
            if first {
                first = false;
            } else {
                write!(f, " ∩ ")?;
            }

            match domain {
                DomainDisplay::Value(value) => write!(f, "{}", value),
                DomainDisplay::Tracker(tracker) => write!(f, "Eq(#{})", tracker),
            }?;
        }

        Ok(())
    }
}
