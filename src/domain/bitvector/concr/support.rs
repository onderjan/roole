use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::LowerHex;
use std::fmt::UpperHex;

use num::BigUint;
use num::Zero;

use crate::domain::bitvector::BitvectorBound;
use crate::domain::bitvector::CBound;
use crate::domain::bitvector::RBound;
use crate::domain::bitvector::bound::compute_u64_mask;
use crate::domain::bitvector::concr::ConcreteBitvector;
use crate::domain::bitvector::concr::ConcreteValue;
use crate::domain::bitvector::concr::OutsideBound;
use crate::domain::bitvector::concr::SignedBitvector;
use crate::domain::bitvector::concr::UnsignedBitvector;
use crate::domain::traits::forward::BExt;
use crate::domain::traits::forward::HwArith;

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub fn from_bool(value: bool, bound: B) -> Self {
        match Self::try_from_u64(value as u64, bound) {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn from_u32(value: u32, bound: B) -> Self {
        match Self::try_from_u64(value.into(), bound) {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn from_u64(value: u64, bound: B) -> Self {
        match Self::try_from_u64(value, bound) {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn try_from_u64(value: u64, bound: B) -> Result<Self, OutsideBound<u64>> {
        // test that the value is within bounds
        if bound.width() < 64 {
            let min_value = 0;
            let max_value = compute_u64_mask(bound.width());

            if value < min_value || value > max_value {
                return Err(OutsideBound {
                    width: bound.width(),
                    value,
                    min_value,
                    max_value,
                });
            }
        }

        Ok(Self {
            value: ConcreteValue::from_u64(value, bound),
            bound,
        })
    }

    pub fn from_big(value: BigUint, bound: B) -> Self {
        let mut above = BigUint::zero();
        above.set_bit(bound.width().into(), true);
        if value >= above {
            panic!("Big value {:?} does not fit width {}", value, bound.width());
        }

        Self {
            value: ConcreteValue::from_big(value, bound),
            bound,
        }
    }

    pub fn bound(&self) -> B {
        self.bound
    }

    pub fn value(self) -> ConcreteValue {
        self.value
    }

    pub fn new_zero(bound: B) -> Self {
        Self {
            value: ConcreteValue::Small(0),
            bound,
        }
    }

    pub fn new_one(bound: B) -> Self {
        // if width is zero, one is the same element as zero
        if bound.width() > 0 {
            Self {
                value: ConcreteValue::Small(1),
                bound,
            }
        } else {
            Self {
                value: ConcreteValue::Small(0),
                bound,
            }
        }
    }

    pub fn single_bit(bit: u32, bound: B) -> Self {
        assert!(bit < bound.width());
        let mut value = ConcreteValue::new_with_zeros(bound);
        value.set_bit(bit, true);

        Self { value, bound }
    }

    pub fn set_sign_bit(&mut self, set_value: bool) {
        if let Some(sign_bit) = self.bound.highest_bit() {
            self.value.set_bit(sign_bit, set_value);
        }
    }

    pub fn set_bit(&mut self, pos: u32, set_value: bool) {
        assert!(pos < self.bound.width());
        self.value.set_bit(pos, set_value);
    }

    pub fn set_bits(&mut self, lo: u32, hi: u32, set_value: bool) {
        assert!(hi < self.bound.width());
        assert!(lo <= hi);
        self.value.set_bits(lo, hi, set_value);
    }

    pub fn from_masked(value: ConcreteValue, bound: B) -> Self {
        let value = value.make_bounded(bound);
        Self { value, bound }
    }

    pub fn try_to_u32(&self) -> Option<u32> {
        self.value.try_to_u32()
    }

    pub fn try_to_u64(&self) -> Option<u64> {
        if self.bound.width() > 64 {
            return None;
        }

        self.value.try_to_u64()
    }

    pub fn try_to_i64(&self) -> Option<i64> {
        let mut result = self.try_to_u64()?;

        if self.is_sign_bit_set() {
            result |= !compute_u64_mask(self.bound.width());
        }
        Some(result as i64)
    }

    pub fn is_sign_bit_set(&self) -> bool {
        if let Some(sign_bit) = self.bound.highest_bit() {
            self.value.is_bit_set(sign_bit)
        } else {
            false
        }
    }

    pub fn is_bit_set(&self, pos: u32) -> bool {
        assert!(pos < self.bound.width());
        self.value.is_bit_set(pos)
    }

    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    pub fn is_one(&self) -> bool {
        if self.bound.width() == 0 {
            return true;
        }

        self.value.is_one()
    }

    pub fn is_nonzero(&self) -> bool {
        !self.is_zero()
    }

    pub fn is_overhalf(&self) -> bool {
        // TODO: make faster
        //self.value == self.bound.sign_bit_mask()
        self == &Self::new_overhalf(self.bound)
    }

    pub fn is_full_mask(&self) -> bool {
        //self.value == self.bound.mask()
        // TODO: make faster
        self == &Self::new_all_ones(self.bound)
    }

    pub fn checked_ilog2(&self) -> Option<u32> {
        self.value.checked_ilog2()
    }

    pub fn is_power_of_two(&self) -> bool {
        self.value.is_power_of_two()
    }

    pub fn trailing_zeros(&self) -> u32 {
        // make sure that the trailing zeros are at most the width
        let width = self.bound.width();
        let result = self.value.trailing_zeros();
        result.min(width)
    }

    pub fn count_ones(&self) -> u32 {
        self.value.count_ones()
    }

    pub fn all_with_bound_iter(bound: B) -> impl Iterator<Item = Self> {
        struct BoundIter<B: BitvectorBound>(Option<ConcreteBitvector<B>>);

        impl<B: BitvectorBound> Iterator for BoundIter<B> {
            type Item = ConcreteBitvector<B>;

            fn next(&mut self) -> Option<Self::Item> {
                match self.0.take() {
                    Some(value) => {
                        let next_value = value.clone().add(ConcreteBitvector::new_one(value.bound));
                        if !next_value.is_zero() {
                            self.0 = Some(next_value);
                        }
                        Some(value)
                    }
                    None => None,
                }
            }
        }

        BoundIter(Some(ConcreteBitvector::new_zero(bound)))
    }

    pub const fn into_unsigned(self) -> UnsignedBitvector<B> {
        UnsignedBitvector::from_bitvector(self)
    }

    pub const fn into_signed(self) -> SignedBitvector<B> {
        SignedBitvector::from_bitvector(self)
    }

    pub fn new_underhalf(bound: B) -> Self {
        // construct all-ones and unset the sign bit
        let mut value = ConcreteValue::new_with_ones(bound);
        if let Some(sign_bit) = bound.highest_bit() {
            value.set_bit(sign_bit, false);
        }
        Self { value, bound }
    }

    pub fn new_overhalf(bound: B) -> Self {
        // construct all-zeros and set the sign bit
        let mut value = ConcreteValue::new_with_zeros(bound);
        if let Some(sign_bit) = bound.highest_bit() {
            value.set_bit(sign_bit, true);
        }
        Self { value, bound }
    }

    pub fn new_all_ones(bound: B) -> Self {
        let value = ConcreteValue::new_with_ones(bound);
        Self { value, bound }
    }

    pub fn new_bool_masked(value: bool, bound: B) -> Self {
        if value {
            Self::new_all_ones(bound)
        } else {
            Self::new_zero(bound)
        }
    }

    pub fn into_runtime_bitvector(self) -> ConcreteBitvector<RBound> {
        ConcreteBitvector {
            bound: RBound::new(self.bound.width()),
            value: self.value,
        }
    }

    pub fn from_ones_width(width: u32, bound: B) -> Self {
        ConcreteBitvector::new_all_ones(RBound::new(width)).uext(bound)
    }

    pub fn num_needed_bits(&self) -> u32 {
        if let Some(ilog2) = self.checked_ilog2() {
            // N + 1 bits are needed to represent a number
            // with the highest set one at position N
            ilog2 + 1
        } else {
            // zero bits are needed to represent zero
            0
        }
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, upper_hex: bool) -> std::fmt::Result {
        let width = self.bound.width();

        write!(f, "0x")?;

        if width == 0 {
            return write!(f, "0");
        }

        let num_nibbles = width.div_ceil(4);

        match &self.value {
            ConcreteValue::Small(value) => {
                for nibble_index in (0..num_nibbles).rev() {
                    let bit_index = nibble_index * 4;
                    let nibble = (value >> bit_index) & 0xF;

                    if upper_hex {
                        write!(f, "{:X}", nibble)?;
                    } else {
                        write!(f, "{:x}", nibble)?;
                    }
                }
            }
            ConcreteValue::Big(elems) => {
                for nibble_index in (0..num_nibbles).rev() {
                    let elem_index = nibble_index / 8;
                    let nibble_index = nibble_index % 8;
                    let bit_index = nibble_index * 4;

                    let elem = elems.get(elem_index as usize).cloned().unwrap_or(0);
                    let nibble = (elem >> bit_index) & 0xF;

                    if upper_hex {
                        write!(f, "{:X}", nibble)?;
                    } else {
                        write!(f, "{:x}", nibble)?;
                    }
                }
            }
        }

        write!(f, "'{}", self.bound.width())?;

        Ok(())
    }
}

impl<const W: u32> ConcreteBitvector<CBound<W>> {
    pub fn from_runtime_bitvector(bitvector: ConcreteBitvector<RBound>) -> Self {
        assert_eq!(bitvector.bound.width(), W);

        Self {
            bound: CBound,
            value: bitvector.value,
        }
    }
}

impl<B: BitvectorBound> Debug for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: print in decimal
        self.format(f, true)
    }
}

impl<B: BitvectorBound> LowerHex for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl<B: BitvectorBound> UpperHex for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}

impl<B: BitvectorBound> Display for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl<T: Display> Display for OutsideBound<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bitvector (width {}) value {} is outside bounds [{},{}]",
            self.width, self.value, self.min_value, self.max_value
        )
    }
}

impl ConcreteBitvector<CBound<1>> {
    pub fn into_bool(self) -> bool {
        !self.is_zero()
    }
}
