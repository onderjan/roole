use std::fmt::{Debug, Display, UpperHex};

use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::{
            BitvectorDisplay, BitvectorDomain, DomainDisplay, ExtendedBitvectorDomain,
            three_valued::InvalidZerosOnes,
        },
        concr::{ConcreteBitvector, SignedBitvector, UnsignedBitvector},
        interval::{SignedInterval, UnsignedInterval},
    },
    traits::{Join, forward::Bitwise},
    value::ThreeValued,
};

use super::ThreeValuedBitvector;

impl<B: BitvectorBound> Join for ThreeValuedBitvector<B> {
    fn join(self, other: &Self) -> Self {
        assert_eq!(self.bound(), other.bound());

        let zeros = self.zeros.bit_or(other.zeros.clone());
        let ones = self.ones.bit_or(other.ones.clone());

        Self::from_zeros_ones(zeros, ones)
    }

    fn apply_join(&mut self, other: &Self) {
        // overwrite self with join result for now
        *self = self.clone().join(other)
    }

    fn contains(&self, contained: &Self) -> bool {
        // rhs zeros must be within our zeros and rhs ones must be within our ones
        // make faster by using the primitives directly
        // and only asserting bound equality in debug mode
        debug_assert_eq!(self.bound(), contained.bound());

        let excessive_rhs_zeros = contained.zeros.to_u64() & (!self.zeros.to_u64());
        let excessive_rhs_ones = contained.ones.to_u64() & (!self.ones.to_u64());
        excessive_rhs_zeros == 0 && excessive_rhs_ones == 0
    }
}

impl<B: BitvectorBound> ThreeValuedBitvector<B> {
    #[must_use]
    pub fn new(value: u64, bound: B) -> Self {
        Self::from_concrete_value(ConcreteBitvector::new(value, bound))
    }

    #[must_use]
    pub fn new_zero(bound: B) -> Self {
        Self::from_concrete_value(ConcreteBitvector::new_zero(bound))
    }

    #[must_use]
    pub fn new_all_ones(bound: B) -> Self {
        Self::from_concrete_value(ConcreteBitvector::new_all_ones(bound))
    }

    #[must_use]
    pub fn from_zeros_ones(zeros: ConcreteBitvector<B>, ones: ConcreteBitvector<B>) -> Self {
        match Self::try_from_zeros_ones(zeros, ones) {
            Ok(ok) => ok,
            Err(_) => panic!("Invalid zeros-ones with some unset bits"),
        }
    }

    pub fn try_from_zeros_ones(
        zeros: ConcreteBitvector<B>,
        ones: ConcreteBitvector<B>,
    ) -> Result<Self, InvalidZerosOnes> {
        assert_eq!(zeros.bound(), ones.bound());

        // the used bits must be set in zeros, ones, or both
        if !Bitwise::bit_or(zeros.clone(), ones.clone()).is_full_mask() {
            return Err(InvalidZerosOnes);
        }
        Ok(Self { zeros, ones })
    }

    pub fn from_concrete_value(value: ConcreteBitvector<B>) -> Self {
        // bit-negate for zeros
        let zeros = Bitwise::bit_not(value.clone());
        // leave as-is for ones
        let ones = value;

        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn is_zeros_sign_bit_set(&self) -> bool {
        self.zeros.is_sign_bit_set()
    }

    #[must_use]
    pub fn is_ones_sign_bit_set(&self) -> bool {
        self.ones.is_sign_bit_set()
    }

    #[must_use]
    pub fn contains_concrete(&self, a: &ConcreteBitvector<B>) -> bool {
        // value zeros must be within our zeros and value ones must be within our ones
        let excessive_rhs_zeros = a.clone().bit_not().bit_and(self.zeros.clone().bit_not());
        let excessive_rhs_ones = a.clone().bit_and(self.ones.clone().bit_not());
        excessive_rhs_zeros.is_zero() && excessive_rhs_ones.is_zero()
    }

    #[must_use]
    pub fn new_unknown(bound: B) -> Self {
        // all zeros and ones set within mask
        let zeros = ConcreteBitvector::new_all_ones(bound);
        let ones = ConcreteBitvector::new_all_ones(bound);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn get_possibly_one_flags(&self) -> &ConcreteBitvector<B> {
        &self.ones
    }

    #[must_use]
    pub fn get_possibly_zero_flags(&self) -> &ConcreteBitvector<B> {
        &self.zeros
    }

    #[must_use]
    pub fn new_value_known(value: ConcreteBitvector<B>, known: ConcreteBitvector<B>) -> Self {
        let unknown = Bitwise::bit_not(known);
        Self::new_value_unknown(value, unknown)
    }

    #[must_use]
    pub fn new_value_unknown(value: ConcreteBitvector<B>, unknown: ConcreteBitvector<B>) -> Self {
        let zeros = Bitwise::bit_or(Bitwise::bit_not(value.clone()), unknown.clone());
        let ones = Bitwise::bit_or(value, unknown);
        Self::from_zeros_ones(zeros, ones)
    }

    #[must_use]
    pub fn unknown_bits(self) -> ConcreteBitvector<B> {
        Bitwise::bit_and(self.zeros.clone(), self.ones.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn all_with_bound_iter(bound: B) -> impl Iterator<Item = Self> {
        let zeros_iter = ConcreteBitvector::<B>::all_with_bound_iter(bound);
        zeros_iter.flat_map(move |zeros| {
            let ones_iter = ConcreteBitvector::<B>::all_with_bound_iter(bound);
            ones_iter.filter_map(move |ones| Self::try_from_zeros_ones(zeros.clone(), ones).ok())
        })
    }

    #[must_use]
    pub fn from_unsigned_interval(interval: UnsignedInterval<B>) -> Self {
        let bound = interval.bound();
        let (min, max) = interval.into_min_max();
        let min = min.cast_bitvector();
        let max = max.cast_bitvector();

        // make positions where min and max agree known
        let xor = min.clone().bit_xor(max);
        let Some(unknown_positions) = xor.to_u64().checked_ilog2() else {
            // min is equal to max
            return Self::from_concrete_value(min);
        };

        let unknown_mask = ConcreteBitvector::from_ones_width(unknown_positions + 1, bound);
        Self::new_value_unknown(min, unknown_mask)
    }

    pub fn write_nonenclosed(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let zeros = self.zeros.to_u64();
        let ones = self.ones.to_u64();

        format_zeros_ones(f, self.bound().width(), zeros, ones, false)
    }

    pub fn set_bit_to_three_valued(&mut self, bit_index: u32, three_valued: ThreeValued) {
        let bit_mask = ConcreteBitvector::new(1 << bit_index, self.bound());

        // TODO: more performant

        match three_valued {
            ThreeValued::False => {
                self.zeros = self.zeros.clone().bit_or(bit_mask.clone());
                self.ones = self.ones.clone().bit_and(bit_mask.bit_not());
            }
            ThreeValued::True => {
                self.zeros = self.zeros.clone().bit_and(bit_mask.clone().bit_not());
                self.ones = self.ones.clone().bit_or(bit_mask);
            }
            ThreeValued::Unknown => {
                self.zeros = self.zeros.clone().bit_or(bit_mask.clone());
                self.ones = self.ones.clone().bit_or(bit_mask);
            }
        }
    }

    pub fn three_valued_from_bit(&self, bit_index: u32) -> ThreeValued {
        let bit_mask = ConcreteBitvector::new(1 << bit_index, self.bound());

        let masked_zeros = self.zeros.clone().bit_and(bit_mask.clone());
        let masked_ones = self.ones.clone().bit_and(bit_mask);

        match (masked_zeros.is_nonzero(), masked_ones.is_nonzero()) {
            (true, true) => ThreeValued::Unknown,
            (true, false) => ThreeValued::False,
            (false, true) => ThreeValued::True,
            (false, false) => panic!("Bit should have a three-valued representation"),
        }
    }

    pub fn unsigned_interval(&self) -> UnsignedInterval<B> {
        UnsignedInterval::new(self.umin(), self.umax())
    }

    pub fn signed_interval(&self) -> SignedInterval<B> {
        SignedInterval::new(self.smin(), self.smax())
    }
}

impl<B: BitvectorBound<SingleBit = B>> ThreeValuedBitvector<B> {
    pub fn from_bools(can_be_false: bool, can_be_true: bool) -> Self {
        let zeros = ConcreteBitvector::from_bool(can_be_false);
        let ones = ConcreteBitvector::from_bool(can_be_true);
        Self::from_zeros_ones(zeros, ones)
    }
}

impl<B: BitvectorBound> BitvectorDomain for ThreeValuedBitvector<B> {
    type Bound = B;

    fn bound(&self) -> Self::Bound {
        // zeros and ones must have the same bound
        self.zeros.bound()
    }

    fn single_value(value: ConcreteBitvector<Self::Bound>) -> Self {
        Self::from_concrete_value(value)
    }

    fn top(bound: B) -> Self {
        Self::new_unknown(bound)
    }

    fn concrete_value(&self) -> Option<ConcreteBitvector<B>> {
        // all bits must be equal
        let nxor = Bitwise::bit_not(Bitwise::bit_xor(self.ones.clone(), self.zeros.clone()));
        if !nxor.is_zero() {
            return None;
        }
        // ones then contain the value
        Some(self.ones.clone())
    }
}

impl<B: BitvectorBound> ExtendedBitvectorDomain for ThreeValuedBitvector<B> {
    type General<X: BitvectorBound> = ThreeValuedBitvector<X>;

    fn meet(self, rhs: &Self) -> Option<Self> {
        let zeros = self.zeros.bit_and(rhs.zeros.clone());
        let ones = self.ones.bit_and(rhs.ones.clone());

        Self::try_from_zeros_ones(zeros, ones).ok()
    }

    fn umin(&self) -> UnsignedBitvector<Self::Bound> {
        // unsigned min value is value of bit-negated zeros (one only where it must be)
        Bitwise::bit_not(self.zeros.clone()).into_unsigned()
    }

    fn umax(&self) -> UnsignedBitvector<Self::Bound> {
        // unsigned max value is value of ones (one everywhere it can be)
        self.ones.clone().into_unsigned()
    }

    fn smin(&self) -> SignedBitvector<B> {
        // take the unsigned minimum
        let mut result = self.umin().cast_bitvector();
        // but the signed value is smaller when the sign bit is one
        // if it is possible to set it to one, set it
        if self.is_ones_sign_bit_set() {
            result.set_sign_bit(true);
        }
        result.into_signed()
    }

    fn smax(&self) -> SignedBitvector<B> {
        // take the unsigned maximum
        let mut result = self.umax().cast_bitvector();
        // but the signed value is bigger when the sign bit is zero
        // if it is possible to set it to zero, set it
        if self.is_zeros_sign_bit_set() {
            result.set_sign_bit(false);
        }
        result.into_signed()
    }

    fn display(&self) -> BitvectorDisplay {
        let domains = vec![DomainDisplay::Value(format!("{}", self))];
        BitvectorDisplay { domains }
    }
}

impl<B: BitvectorBound> Debug for ThreeValuedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let zeros = self.zeros.to_u64();
        let ones = self.ones.to_u64();

        format_zeros_ones(f, self.bound().width(), zeros, ones, true)
    }
}

impl<B: BitvectorBound> UpperHex for ThreeValuedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // just do the normal debug formatting
        <Self as Debug>::fmt(self, f)
    }
}

impl<B: BitvectorBound> Display for ThreeValuedBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

pub fn format_zeros_ones(
    f: &mut std::fmt::Formatter<'_>,
    bit_width: u32,
    zeros: u64,
    ones: u64,
    enclosed: bool,
) -> std::fmt::Result {
    let nxor = !(ones ^ zeros);
    if nxor == 0 {
        // concrete value
        return write!(f, "{:?}", ones);
    }

    if enclosed {
        write!(f, "\"")?;
    }
    for little_k in 0..bit_width {
        let big_k = bit_width - little_k - 1;
        let zero = (zeros >> (big_k as usize)) & 1 != 0;
        let one = (ones >> (big_k as usize)) & 1 != 0;
        let c = match (zero, one) {
            (true, true) => 'X',
            (true, false) => '0',
            (false, true) => '1',
            (false, false) => 'V',
        };
        write!(f, "{}", c)?;
    }
    if enclosed { write!(f, "\"") } else { Ok(()) }
}
