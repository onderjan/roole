use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Neg, Not, Sub};

use crate::domain::bitvector::{BitvectorBound, bound::compute_u64_mask, concr::ConcreteValue};

impl Not for ConcreteValue {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            ConcreteValue::Small(value) => ConcreteValue::Small(!value),
            ConcreteValue::Big(values) => {
                ConcreteValue::Big(Box::from_iter(values.iter().map(|e| !e)))
            }
        }
    }
}

impl BitAnd for ConcreteValue {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        self.bitwise_fn(rhs, |a, b, _| (a & b, ()), ())
    }
}

impl BitOr for ConcreteValue {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.bitwise_fn(rhs, |a, b, _| (a | b, ()), ())
    }
}

impl BitXor for ConcreteValue {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        self.bitwise_fn(rhs, |a, b, _| (a ^ b, ()), ())
    }
}

impl Neg for ConcreteValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            ConcreteValue::Small(value) => ConcreteValue::Small(0u64.wrapping_sub(value)),
            ConcreteValue::Big(words) => {
                let mut borrow = false;
                ConcreteValue::Big(Box::from_iter(words.iter().map(|word| {
                    let (value, new_borrow) = 0u64.borrowing_sub(*word, borrow);
                    borrow = new_borrow;
                    value
                })))
            }
        }
    }
}

impl Add for ConcreteValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.bitwise_fn(rhs, |a, b, carry| a.carrying_add(b, carry), false)
    }
}

impl Sub for ConcreteValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.bitwise_fn(rhs, |a, b, borrow| a.borrowing_sub(b, borrow), false)
    }
}

impl Mul for ConcreteValue {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ConcreteValue::Small(lhs), ConcreteValue::Small(rhs)) => {
                ConcreteValue::Small(lhs.wrapping_mul(rhs))
            }
            (ConcreteValue::Big(lhs), ConcreteValue::Big(rhs)) => {
                let num_words = lhs.len();
                assert_eq!(num_words, rhs.len());
                let mut result = vec![0u64; num_words].into_boxed_slice();

                // classic quadratic multiplication
                for j in 0..num_words {
                    let mut carry = 0;
                    for i in 0..(num_words - j) {
                        (result[j + i], carry) =
                            lhs[i].carrying_mul_add(rhs[j], result[j + i], carry);
                    }
                }
                ConcreteValue::Big(result)
            }
            _ => panic!("Values must have same storage"),
        }
    }
}

impl ConcreteValue {
    pub fn new_with_zeros<B: BitvectorBound>(bound: B) -> Self {
        let width = bound.width();
        let num_words = width.div_ceil(64);

        if num_words <= 1 {
            return Self::Small(0);
        }

        Self::Big(vec![u64::MAX; num_words.try_into().unwrap()].into_boxed_slice())
    }

    pub fn new_with_ones<B: BitvectorBound>(bound: B) -> Self {
        let width = bound.width();
        let num_words = width.div_ceil(64);

        let partial_width = width % 64;
        let partial_mask = compute_u64_mask(partial_width);

        if num_words <= 1 {
            return Self::Small(partial_mask);
        }

        let mut words = vec![u64::MAX; num_words.try_into().unwrap()].into_boxed_slice();

        // mask the last word if needed
        if partial_width != 0 {
            let last_word = words.last_mut().unwrap();
            *last_word &= partial_mask;
        }

        Self::Big(words)
    }

    pub fn len(&self) -> u32 {
        match self {
            ConcreteValue::Small(_) => 1,
            ConcreteValue::Big(items) => items.len().try_into().unwrap(),
        }
    }

    pub fn check_bound<B: BitvectorBound>(&self, bound: B) {
        assert_eq!(self.len(), bound.word_len())
    }

    fn bitwise_fn<A: Copy>(self, rhs: Self, fun: fn(u64, u64, A) -> (u64, A), mut acc: A) -> Self {
        match (self, rhs) {
            (ConcreteValue::Small(lhs), ConcreteValue::Small(rhs)) => {
                let (value, _new_acc) = fun(lhs, rhs, acc);
                ConcreteValue::Small(value)
            }
            (ConcreteValue::Big(lhs), ConcreteValue::Big(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());
                ConcreteValue::Big(Box::from_iter(lhs.iter().zip(rhs).map(|(lhs, rhs)| {
                    let (value, new_acc) = fun(*lhs, rhs, acc);
                    acc = new_acc;

                    value
                })))
            }
            _ => panic!("Values must have same storage"),
        }
    }

    pub(super) fn make_bounded<B: BitvectorBound>(self, bound: B) -> Self {
        let width = bound.width();

        let num_new_words = width.div_ceil(64);
        if num_new_words == 0 {
            return ConcreteValue::Small(0);
        }

        let partial_width = width % 64;
        let partial_mask = compute_u64_mask(partial_width);

        match self {
            ConcreteValue::Small(value) => {
                if num_new_words == 1 {
                    // keep the value small
                    ConcreteValue::Small(value & partial_mask)
                } else {
                    // make the value big, pad with zero words
                    let mut words =
                        vec![0u64; num_new_words.try_into().unwrap()].into_boxed_slice();
                    words[0] = value;
                    ConcreteValue::Big(words)
                }
            }
            ConcreteValue::Big(words) => {
                assert!(words.len() > 1);
                if num_new_words == 1 {
                    // take the first word
                    // the partial mask is on some word above, no need to consider it
                    return ConcreteValue::Small(words[0]);
                }
                // resize according to the number of words
                let num_old_words: u32 = words.len().try_into().unwrap();

                let mut words = if num_new_words != num_old_words {
                    // resize, pad with zeros if needed
                    let mut vec = words.into_vec();
                    vec.resize(num_new_words.try_into().unwrap(), 0);
                    vec.into_boxed_slice()
                } else {
                    words
                };

                // mask the last word if needed
                if partial_width != 0 {
                    let last_word = words.last_mut().unwrap();
                    *last_word &= partial_mask;
                }

                ConcreteValue::Big(words)
            }
        }
    }

    pub fn is_bit_set(&self, bit: u32) -> bool {
        let word_index: usize = (bit / 64).try_into().unwrap();
        let bit_in_word = bit % 64;

        let word = match self {
            ConcreteValue::Small(value) => value,
            ConcreteValue::Big(words) => &words[word_index],
        };

        word & (1 << bit_in_word) != 0
    }

    pub fn set_bit(&mut self, bit: u32, set_value: bool) {
        let word_index: usize = (bit / 64).try_into().unwrap();
        let bit_in_word = bit % 64;

        let word = match self {
            ConcreteValue::Small(value) => value,
            ConcreteValue::Big(words) => &mut words[word_index],
        };

        if set_value {
            *word |= 1 << bit_in_word;
        } else {
            *word &= !(1 << bit_in_word);
        }
    }

    pub fn try_to_u32(&self) -> Option<u32> {
        let word = match self {
            ConcreteValue::Small(small) => *small,
            ConcreteValue::Big(words) => {
                // all words other than zero word must be zero
                for i in 1..words.len() {
                    if words[i] != 0 {
                        return None;
                    }
                }
                words[0]
            }
        };

        word.try_into().ok()
    }
}
