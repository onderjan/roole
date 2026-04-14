use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::LowerHex;
use std::fmt::UpperHex;

use crate::domain::bitvector::BitvectorBound;
use crate::domain::bitvector::CBound;
use crate::domain::bitvector::RBound;
use crate::domain::bitvector::bound::compute_u64_mask;
use crate::domain::bitvector::concr::ConcreteBitvector;
use crate::domain::bitvector::concr::ConcreteValue;
use crate::domain::bitvector::concr::OutsideBound;
use crate::domain::bitvector::concr::SignedBitvector;
use crate::domain::bitvector::concr::UnsignedBitvector;
use crate::domain::traits::forward::HwArith;

impl<B: BitvectorBound> ConcreteBitvector<B> {
    pub fn new(value: u64, bound: B) -> Self {
        match Self::try_new(value, bound) {
            Ok(ok) => ok,
            Err(err) => panic!("{}", err),
        }
    }

    pub fn try_new(value: u64, bound: B) -> Result<Self, OutsideBound<u64>> {
        // test that the value is within bounds
        let min_value = 0;
        let max_value = bound.mask();

        if value < min_value || value > max_value {
            return Err(OutsideBound {
                width: bound.width(),
                value,
                min_value,
                max_value,
            });
        }

        Ok(Self {
            value: ConcreteValue::Small(value),
            bound,
        })
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

    pub fn from_masked(value: ConcreteValue, bound: B) -> Self {
        let value = value.make_bounded(bound);
        Self { value, bound }
    }

    /*pub fn from_masked_u64(value: u64, bound: B) -> Self {
        let value = value & bound.mask();
        Self { value, bound }
    }*/

    pub fn try_to_u32(&self) -> Option<u32> {
        self.value.try_to_u32()
    }

    pub fn to_u64(&self) -> u64 {
        // TODO: never convert to u64
        if self.bound.width() > 64 {
            panic!("Bound too big to convert");
        }

        match &self.value {
            ConcreteValue::Small(value) => *value,
            ConcreteValue::Big(items) => items.first().cloned().unwrap_or(0),
        }
    }

    pub fn to_i64(&self) -> i64 {
        // TODO: never convert to u64
        if self.bound.width() > 64 {
            panic!("Bound too big to convert");
        }

        let mut result = match &self.value {
            ConcreteValue::Small(value) => *value,
            ConcreteValue::Big(items) => items.first().cloned().unwrap_or(0),
        };

        let sign_bit_mask = self.bound.sign_bit_mask();
        if result & sign_bit_mask != 0 {
            // add signed extension
            result |= !self.bound.mask();
        }
        result as i64
    }

    pub fn is_sign_bit_set(&self) -> bool {
        if let Some(sign_bit) = self.bound.highest_bit() {
            self.value.is_bit_set(sign_bit)
        } else {
            false
        }
    }

    pub fn is_zero(&self) -> bool {
        match &self.value {
            ConcreteValue::Small(value) => *value == 0,
            ConcreteValue::Big(elems) => elems.iter().all(|e| *e == 0),
        }
    }

    pub fn is_nonzero(&self) -> bool {
        !self.is_zero()
    }

    pub fn is_one(&self) -> bool {
        if self.bound.width() == 0 {
            return true;
        }

        match &self.value {
            ConcreteValue::Small(value) => *value == 0,
            ConcreteValue::Big(elems) => {
                let mut first = true;
                for e in elems {
                    let expected = if first { 1 } else { 0 };
                    if *e != expected {
                        return false;
                    }
                    first = false;
                }
                true
            }
        }
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

    pub fn all_with_bound_iter(bound: B) -> impl Iterator<Item = Self> {
        //(0..=bound.mask()).map(move |value| Self { bound, value })
        todo!("All with bound iter");
        std::iter::empty()
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
        let value = compute_u64_mask(width);

        Self::new(value, bound)
    }

    pub fn num_needed_bits(&self) -> u32 {
        todo!("Num needed bits")
        /*if let Some(ilog2) = self.value.checked_ilog2() {
            // N + 1 bits are needed to represent a number
            // with the highest set one at position N
            ilog2 + 1
        } else {
            // zero bits are needed to represent zero
            0
        }*/
    }

    fn format(&self, f: &mut std::fmt::Formatter<'_>, upper_hex: bool) -> std::fmt::Result {
        let width = self.bound.width();

        if width == 0 {
            return write!(f, "0x0");
        }

        let last_nibble = width.div_ceil(4);

        match &self.value {
            ConcreteValue::Small(value) => {
                write!(f, "0x")?;

                for nibble_index in (0..=last_nibble).rev() {
                    let nibble = (value >> nibble_index) & 0xF;

                    if upper_hex {
                        write!(f, "{:X}", nibble)?;
                    } else {
                        write!(f, "{:x}", nibble)?;
                    }
                }
            }
            ConcreteValue::Big(elems) => {
                for nibble_index in (0..=last_nibble).rev() {
                    let elem_index = nibble_index / 8;
                    let nibble_index = nibble_index % 8;

                    let elem = elems.get(elem_index as usize).cloned().unwrap_or(0);
                    let nibble = (elem >> nibble_index) & 0xF;

                    if upper_hex {
                        write!(f, "{:X}", nibble)?;
                    } else {
                        write!(f, "{:x}", nibble)?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<B: BitvectorBound<SingleBit = B>> ConcreteBitvector<B> {
    pub fn from_bool(value: bool) -> Self {
        Self::new(value as u64, B::single_bit_bound())
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
