use std::fmt::Debug;

use super::{
    super::{
        BitvectorBound,
        concr::{ConcreteBitvector, SignedBitvector},
    },
    SignlessInterval,
};

/// A signed interval with a minimum and a maximum value.
///
/// It is required that min <= max, which means the interval
/// does not support wrapping nor representing an empty set.
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct SignedInterval<B: BitvectorBound> {
    min: SignedBitvector<B>,
    max: SignedBitvector<B>,
}

impl<B: BitvectorBound> SignedInterval<B> {
    pub fn new(min: SignedBitvector<B>, max: SignedBitvector<B>) -> Self {
        // comparison will panic on different bound values
        assert!(min <= max);
        Self { min, max }
    }

    pub fn min(&self) -> &SignedBitvector<B> {
        &self.min
    }

    pub fn max(&self) -> &SignedBitvector<B> {
        &self.max
    }

    pub fn bound(&self) -> B {
        // the bound must be the same for min and max
        self.min.bound()
    }

    pub fn from_value(value: SignedBitvector<B>) -> Self {
        Self {
            min: value.clone(),
            max: value,
        }
    }

    pub fn try_into_signless(self) -> Option<SignlessInterval<B>> {
        let min = self.min.cast_bitvector();
        let max = self.max.cast_bitvector();
        if min.is_sign_bit_set() == max.is_sign_bit_set() {
            Some(SignlessInterval::new(min, max))
        } else {
            None
        }
    }

    pub fn into_signless_halves(
        self,
    ) -> (Option<SignlessInterval<B>>, Option<SignlessInterval<B>>) {
        assert!(self.bound().width() > 0);

        let zero = ConcreteBitvector::new_zero(self.bound()).into_signed();

        if self.min >= zero {
            // only nonnegative interval
            let interval =
                SignlessInterval::new(self.min.cast_bitvector(), self.max.cast_bitvector());
            return (None, Some(interval));
        }
        // negative interval exists
        if self.max < zero {
            // only negative interval
            let interval =
                SignlessInterval::new(self.min.cast_bitvector(), self.max.cast_bitvector());
            return (Some(interval), None);
        }
        // both intervals exist
        let minus_one = ConcreteBitvector::new_all_ones(self.bound());

        let negative_interval = SignlessInterval::new(self.min.cast_bitvector(), minus_one);
        let nonnegative_interval =
            SignlessInterval::new(zero.cast_bitvector(), self.max.cast_bitvector());
        (Some(negative_interval), Some(nonnegative_interval))
    }

    pub fn ext<X: BitvectorBound>(self, new_bound: X) -> SignedInterval<X> {
        if self.min == self.max {
            // clearly, we can extend
            let ext_value = self.min.ext(new_bound);
            return SignedInterval {
                min: ext_value.clone(),
                max: ext_value,
            };
        }

        // if we narrow the interval and disregarded a bound, saturate
        let mut ext_min: SignedBitvector<X> = self.min.clone().ext(new_bound);
        let mut ext_max: SignedBitvector<X> = self.max.clone().ext(new_bound);

        let self_bound = self.min.bound();
        assert_eq!(self_bound, self.max.bound());

        let min_diff = self.min - ext_min.clone().ext(self_bound);
        let max_diff = self.max - ext_max.clone().ext(self_bound);

        if min_diff != max_diff {
            // we disregarded a bound, saturate
            ext_min = ConcreteBitvector::new_overhalf(new_bound).into_signed();
            ext_max = ConcreteBitvector::new_underhalf(new_bound).into_signed();
        }
        SignedInterval {
            min: ext_min,
            max: ext_max,
        }
    }

    #[allow(dead_code)]
    pub fn contains_value(&self, value: &SignedBitvector<B>) -> bool {
        &self.min <= value && value <= &self.max
    }
}

impl<B: BitvectorBound> Debug for SignedInterval<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}
