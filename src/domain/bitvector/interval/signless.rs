use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use super::{
    super::{BitvectorBound, CBound, RBound, concr::ConcreteBitvector},
    SignedInterval, UnsignedInterval, WrappingInterval,
};

/// A signless interval with a minimum and a maximum value.
///
/// It is required that the signless interval has the minimum
/// and maximum value in the same half-plane.
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignlessInterval<B: BitvectorBound> {
    min: ConcreteBitvector<B>,
    max: ConcreteBitvector<B>,
}

impl<B: BitvectorBound> SignlessInterval<B> {
    pub fn new(min: ConcreteBitvector<B>, max: ConcreteBitvector<B>) -> Self {
        assert_eq!(min.is_sign_bit_set(), max.is_sign_bit_set());
        // comparison will panic on different bound values
        // we must convert to either signed or unsigned to have comparisons available
        // since the sign bits are the same, it is irrelevant which one
        let min = min.into_unsigned();
        let max = max.into_unsigned();
        assert!(min <= max);
        let min = min.cast_bitvector();
        let max = max.cast_bitvector();
        Self { min, max }
    }

    pub fn from_value(value: ConcreteBitvector<B>) -> Self {
        Self {
            min: value.clone(),
            max: value,
        }
    }

    pub fn bound(&self) -> B {
        // bounds must be the same for min and max
        self.min.bound()
    }

    pub fn is_sign_bit_set(&self) -> bool {
        // both min and max must have the same value of sign bit
        self.min.is_sign_bit_set()
    }

    pub(crate) fn new_full_near_halfplane(bound: B) -> Self {
        Self {
            min: ConcreteBitvector::<B>::new_zero(bound),
            max: ConcreteBitvector::<B>::new_underhalf(bound),
        }
    }

    pub(crate) fn new_full_far_halfplane(bound: B) -> Self {
        Self {
            min: ConcreteBitvector::<B>::new_overhalf(bound),
            max: ConcreteBitvector::<B>::new_all_ones(bound),
        }
    }

    pub fn contains_value(&self, value: &ConcreteBitvector<B>) -> bool {
        // we can use either interpretation
        let value = value.clone().into_unsigned();
        self.min.clone().into_unsigned() <= value && value <= self.max.clone().into_unsigned()
    }

    pub fn concrete_value(&self) -> Option<ConcreteBitvector<B>> {
        if self.min == self.max {
            return Some(self.min.clone());
        }
        None
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        assert_eq!(self.bound(), other.bound());
        assert_eq!(self.min.is_sign_bit_set(), other.min.is_sign_bit_set());
        let min = self.min.into_unsigned().max(other.min.into_unsigned());
        let max = self.max.into_unsigned().min(other.max.into_unsigned());
        if min <= max {
            Some(Self {
                min: min.cast_bitvector(),
                max: max.cast_bitvector(),
            })
        } else {
            None
        }
    }

    pub fn union(self, other: Self) -> Self {
        assert_eq!(self.bound(), other.bound());
        assert_eq!(self.min.is_sign_bit_set(), other.min.is_sign_bit_set());
        Self {
            min: self
                .min
                .into_unsigned()
                .min(other.min.into_unsigned())
                .cast_bitvector(),
            max: self
                .max
                .into_unsigned()
                .max(other.max.into_unsigned())
                .cast_bitvector(),
        }
    }

    pub fn union_opt(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (None, None) => None,
            (None, Some(b)) => Some(b),
            (Some(a), None) => Some(a),
            (Some(a), Some(b)) => Some(a.union(b)),
        }
    }

    pub fn min(&self) -> &ConcreteBitvector<B> {
        &self.min
    }
    pub fn max(&self) -> &ConcreteBitvector<B> {
        &self.max
    }

    pub fn into_wrapping(self) -> WrappingInterval<B> {
        WrappingInterval::new(self.min, self.max)
    }

    pub fn into_unsigned(self) -> UnsignedInterval<B> {
        UnsignedInterval::new(self.min.into_unsigned(), self.max.into_unsigned())
    }

    pub fn into_signed(self) -> SignedInterval<B> {
        SignedInterval::new(self.min.into_signed(), self.max.into_signed())
    }

    #[allow(dead_code)]
    pub fn all_with_bound_iter(bound: B, far: bool) -> impl Iterator<Item = Self> {
        let min_iter = ConcreteBitvector::<B>::all_with_bound_iter(bound);
        min_iter
            .flat_map(move |min| {
                if min.is_sign_bit_set() != far {
                    return None;
                }

                let max_iter = ConcreteBitvector::<B>::all_with_bound_iter(bound);

                let result = max_iter.flat_map(move |max| {
                    if max.is_sign_bit_set() != far {
                        return None;
                    }
                    if min.to_u64() > max.to_u64() {
                        return None;
                    }

                    Some(SignlessInterval::new(min.clone(), max))
                });
                Some(result)
            })
            .flatten()
    }

    #[allow(dead_code)]
    pub fn contains(&self, other: &Self) -> bool {
        if self.min.is_sign_bit_set() != other.min.is_sign_bit_set() {
            return false;
        }
        self.min.clone().into_unsigned() <= other.min.clone().into_unsigned()
            && other.max.clone().into_unsigned() <= self.max.clone().into_unsigned()
    }
}

impl<B: BitvectorBound> Debug for SignlessInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}

impl<B: BitvectorBound> Display for SignlessInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<const W: u32> SignlessInterval<CBound<W>> {
    pub(crate) fn from_runtime(value: SignlessInterval<RBound>) -> Self {
        Self {
            min: ConcreteBitvector::from_runtime_bitvector(value.min),
            max: ConcreteBitvector::from_runtime_bitvector(value.max),
        }
    }

    pub(crate) fn into_runtime(self) -> SignlessInterval<RBound> {
        SignlessInterval {
            min: self.min.into_runtime_bitvector(),
            max: self.max.into_runtime_bitvector(),
        }
    }
}
