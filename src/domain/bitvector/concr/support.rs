use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::LowerHex;
use std::fmt::UpperHex;

use crate::domain::bitvector::BitvectorBound;
use crate::domain::bitvector::CBound;
use crate::domain::bitvector::RBound;
use crate::domain::bitvector::bound::compute_u64_mask;
use crate::domain::bitvector::concr::ConcreteBitvector;
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

        Ok(Self { value, bound })
    }

    pub fn bound(self) -> B {
        self.bound
    }

    pub fn zero(bound: B) -> Self {
        Self { value: 0, bound }
    }

    pub fn one(bound: B) -> Self {
        // mask by bound to support zero-sized bitvectors
        let one = 1 & bound.mask();
        Self { value: one, bound }
    }

    pub fn bit_mask(bound: B) -> Self {
        Self {
            value: bound.mask(),
            bound,
        }
    }

    pub fn sign_bit_mask(bound: B) -> Self {
        Self {
            value: bound.sign_bit_mask(),
            bound,
        }
    }

    pub fn from_masked_u64(value: u64, bound: B) -> Self {
        let value = value & bound.mask();
        Self { value, bound }
    }

    pub fn to_u64(self) -> u64 {
        self.value
    }

    pub fn to_i64(self) -> i64 {
        let mut result = self.value;
        let sign_bit_mask = self.bound.sign_bit_mask();
        if self.value & sign_bit_mask != 0 {
            // add signed extension
            result |= !self.bound.mask();
        }
        result as i64
    }

    pub fn is_sign_bit_set(self) -> bool {
        self.value & self.bound.sign_bit_mask() != 0
    }

    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    pub fn is_nonzero(&self) -> bool {
        self.value != 0
    }

    pub fn is_one(&self) -> bool {
        if self.bound.width() == 0 {
            true
        } else {
            self.value == 1
        }
    }

    pub fn is_overhalf(&self) -> bool {
        self.value == self.bound.sign_bit_mask()
    }

    pub fn is_full_mask(&self) -> bool {
        self.value == self.bound.mask()
    }

    pub fn all_with_bound_iter(bound: B) -> impl Iterator<Item = Self> {
        (0..=bound.mask()).map(move |value| Self { bound, value })
    }

    pub const fn as_unsigned(self) -> UnsignedBitvector<B> {
        UnsignedBitvector::from_bitvector(self)
    }

    pub const fn as_signed(self) -> SignedBitvector<B> {
        SignedBitvector::from_bitvector(self)
    }

    pub fn new_umin(bound: B) -> Self {
        // this is just zero
        Self::zero(bound)
    }

    pub fn new_underhalf(bound: B) -> Self {
        let value = bound.mask() ^ bound.sign_bit_mask();
        Self::from_masked_u64(value, bound)
    }

    pub fn new_overhalf(bound: B) -> Self {
        let value = bound.sign_bit_mask();
        Self::from_masked_u64(value, bound)
    }

    pub fn new_umax(bound: B) -> Self {
        let value = bound.mask();
        Self::from_masked_u64(value, bound)
    }

    pub fn as_runtime_bitvector(self) -> ConcreteBitvector<RBound> {
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
        if let Some(ilog2) = self.value.checked_ilog2() {
            // N + 1 bits are needed to represent a number
            // with the highest set one at position N
            ilog2 + 1
        } else {
            // zero bits are needed to represent zero
            0
        }
    }

    pub fn modular_inverse(self) -> Option<Self> {
        // TODO: more general computation of modular inverse
        let a = self.to_u64().into();
        let modulus = 1i128 << self.bound.width();

        let (g, x, _) = Self::extended_euclidean(a, modulus);
        if g != 1 {
            return None;
        }

        let inverse = ((x % modulus) + modulus) % modulus;
        let Ok(inverse) = inverse.try_into() else {
            panic!("Modular inverse does not fit in u64");
        };
        let inverse = Self::new(inverse, self.bound);

        assert_eq!(inverse.mul(self), ConcreteBitvector::one(self.bound));

        Some(inverse)
    }

    fn extended_euclidean(a: i128, b: i128) -> (i128, i128, i128) {
        if a == 0 || b == 0 {
            panic!("Extended Euclidean algorithm cannot be used with zero");
        }
        eprintln!("Extended euclidean: {}, {}", a, b);

        let mut x = 1;
        let mut y = 0;
        let mut x1 = 0;
        let mut y1 = 1;
        let mut a1 = a;
        let mut b1 = b;
        loop {
            if b1 == 0 {
                let gcd = x * a + y * b;
                eprintln!("GCD: {} = {}*{} + {}*{}", gcd, x, a, y, b);

                return (gcd, x, y);
            }

            let q = a1 / b1;
            (x, x1) = (x1, x - q * x1);
            (y, y1) = (y1, y - q * y1);
            (a1, b1) = (b1, a1 - q * b1);
        }
    }
}

fn extended_gcd(a: u64, b: u64) -> (u64, u64, u64) {
    if a == 0 {
        return (b, 0, 1);
    }
    let (g, x, y) = extended_gcd(b % a, a);
    let next = y - (b / a) * x;
    (g, next, x)
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
        // ignore bound
        std::fmt::Debug::fmt(&self.value, f)
    }
}

impl<B: BitvectorBound> LowerHex for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore bound
        std::fmt::LowerHex::fmt(&self.value, f)
    }
}

impl<B: BitvectorBound> UpperHex for ConcreteBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore bound
        std::fmt::UpperHex::fmt(&self.value, f)
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
