use std::cmp::Ordering;

use crate::domain::bitvector::{BitvectorBound, bound::compute_u64_mask, concr::ConcreteValue};

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

    pub fn uni_upwards<A: Copy>(self, fun: fn(u64, A) -> (u64, A), mut acc: A) -> Self {
        match self {
            ConcreteValue::Small(small) => {
                let (value, _new_acc) = fun(small, acc);
                ConcreteValue::Small(value)
            }
            ConcreteValue::Big(words) => {
                ConcreteValue::Big(Box::from_iter(words.iter().map(|word| {
                    let (value, new_acc) = fun(*word, acc);
                    acc = new_acc;
                    value
                })))
            }
        }
    }

    pub fn bi_upwards<A: Copy>(
        self,
        rhs: Self,
        fun: fn(u64, u64, A) -> (u64, A),
        mut acc: A,
    ) -> Self {
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

    pub fn bi_small(self, rhs: Self, fun: impl Fn(u64, u64) -> u64) -> Self {
        match (self, rhs) {
            (ConcreteValue::Small(lhs), ConcreteValue::Small(rhs)) => {
                ConcreteValue::Small(fun(lhs, rhs))
            }
            (ConcreteValue::Big(_), ConcreteValue::Big(_)) => {
                // TODO: implement all operations for arbitrary sizes
                panic!("Operation only supported for at most 64-bit bitvectors");
            }
            _ => panic!("Values must have same storage"),
        }
    }

    pub fn mul(self, rhs: Self) -> Self {
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

    pub fn unbounded_shl(self, rhs: Self) -> Self {
        let Some(rhs) = rhs.try_to_u32() else {
            // shift too big, the result is zero
            return match self {
                ConcreteValue::Small(_) => ConcreteValue::Small(0),
                ConcreteValue::Big(lhs) => {
                    ConcreteValue::Big(vec![0u64; lhs.len()].into_boxed_slice())
                }
            };
        };

        match self {
            ConcreteValue::Small(lhs) => ConcreteValue::Small(lhs.unbounded_shl(rhs)),
            ConcreteValue::Big(lhs) => Self::unbounded_shl_big(&lhs, rhs),
        }
    }

    fn unbounded_shl_big(lhs: &[u64], rhs: u32) -> Self {
        let num_words = lhs.len();
        let mut result = vec![0u64; num_words].into_boxed_slice();
        // usually, each result word will combine two words from lhs
        // those words will have indices lower or equal to the result word

        // go from the highest to lowest so we can break once there are no more bits to process
        for i in (0..num_words.try_into().unwrap()).rev() {
            let i_usize: usize = i.try_into().unwrap();
            let lowest_dest_bit: u32 = i * 64;
            let highest_dest_bit = lowest_dest_bit + 63;

            let Some(highest_src_bit) = highest_dest_bit.checked_sub(rhs) else {
                // the highest source bit is under the available bits, we can break
                break;
            };

            let Some(lowest_src_bit) = lowest_dest_bit.checked_sub(rhs) else {
                // the lowest source bit is under the available bits
                // the highest source bit must be within the lowest source word
                // move from there
                let mask = compute_u64_mask(highest_src_bit);
                let src_value = lhs[0] & mask;
                // we have to move the source value up to account for the bits under
                let num_under = rhs - lowest_dest_bit;
                result[i_usize] = src_value << num_under;
                // we will have the source bits under available bits in next iterations, break
                break;
            };

            // both the highest and lowest source bit are available
            let word_lo = lowest_src_bit / 64;
            let word_hi = highest_dest_bit / 64;
            let word_lo: usize = word_lo.try_into().unwrap();
            let word_hi: usize = word_hi.try_into().unwrap();
            if word_lo == word_hi {
                // just copy the word
                result[i_usize] = lhs[word_lo];
            } else {
                // the lower part is in a word one lower than the higher part
                let bit_lo = lowest_src_bit % 64;
                let bit_hi = highest_src_bit % 64;

                let width_lo = 64 - bit_lo;

                // make source masks for bits at and above bit_lo and at and below bit_hi
                // note that bit_hi must be below 63
                let mask_lo = !compute_u64_mask(bit_lo);
                let mask_hi = compute_u64_mask(bit_hi + 1);

                // combine values
                let value_lo = (lhs[word_lo] & mask_lo) >> bit_lo;
                let value_hi = (lhs[word_hi] & mask_hi) << width_lo;
                result[i_usize] = value_lo | value_hi;
            }
        }

        ConcreteValue::Big(result)
    }

    pub fn unbounded_shr(self, rhs: Self) -> Self {
        let Some(rhs) = rhs.try_to_u32() else {
            // shift too big, the result is zero
            return match self {
                ConcreteValue::Small(_) => ConcreteValue::Small(0),
                ConcreteValue::Big(lhs) => {
                    ConcreteValue::Big(vec![0u64; lhs.len()].into_boxed_slice())
                }
            };
        };

        match self {
            ConcreteValue::Small(lhs) => ConcreteValue::Small(lhs.unbounded_shr(rhs)),
            ConcreteValue::Big(lhs) => Self::unbounded_shl_big(&lhs, rhs),
        }
    }

    fn unbounded_shr_big(lhs: &[u64], rhs: u32) -> Self {
        let num_words = lhs.len();
        let mut result = vec![0u64; num_words].into_boxed_slice();
        // usually, each result word will combine two words from lhs
        // those words will have indices greater or equal to the result word

        // go from the lowest to highest so we can break once there are no more bits to process
        for i in 0..num_words.try_into().unwrap() {
            let i_usize: usize = i.try_into().unwrap();
            let lowest_dest_bit: u32 = i * 64;
            let highest_dest_bit = lowest_dest_bit + 63;

            let lowest_src_bit = lowest_dest_bit + rhs;
            let highest_src_bit = highest_dest_bit + rhs;

            let word_lo = lowest_src_bit / 64;
            let word_hi = highest_dest_bit / 64;
            let word_lo: usize = word_lo.try_into().unwrap();
            let word_hi: usize = word_hi.try_into().unwrap();
            if word_lo == word_hi {
                // just copy the word or fill with zero if above the available
                result[i_usize] = lhs.get(word_lo).copied().unwrap_or(0);
            } else {
                // the lower part is in a word one lower than the higher part
                let bit_lo = lowest_src_bit % 64;
                let bit_hi = highest_src_bit % 64;

                let width_lo = 64 - bit_lo;

                // make source masks for bits at and above bit_lo and at and below bit_hi
                // note that bit_hi must be below 63
                let mask_lo = !compute_u64_mask(bit_lo);
                let mask_hi = compute_u64_mask(bit_hi + 1);

                // combine values, fill with zero if not available
                let value_lo = (lhs.get(word_lo).copied().unwrap_or(0) & mask_lo) >> bit_lo;
                let value_hi = (lhs.get(word_hi).copied().unwrap_or(0) & mask_hi) << width_lo;
                result[i_usize] = value_lo | value_hi;
            }
        }

        ConcreteValue::Big(result)
    }

    pub fn unsigned_cmp(&self, rhs: &Self) -> Ordering {
        match (self, rhs) {
            (ConcreteValue::Small(lhs), ConcreteValue::Small(rhs)) => lhs.cmp(rhs),
            (ConcreteValue::Big(lhs), ConcreteValue::Big(rhs)) => {
                assert_eq!(lhs.len(), rhs.len());

                // the comparison is done from the highest to lowest element
                for (lhs, rhs) in lhs.iter().zip(rhs).rev() {
                    match lhs.cmp(rhs) {
                        Ordering::Less => {
                            return Ordering::Less;
                        }
                        Ordering::Equal => {
                            // continue comparing
                        }
                        Ordering::Greater => {
                            return Ordering::Greater;
                        }
                    }
                }
                // everything equal
                Ordering::Equal
            }
            _ => panic!("Values must have same storage"),
        }
    }

    pub fn checked_ilog2(&self) -> Option<u32> {
        match self {
            ConcreteValue::Small(value) => value.checked_ilog2(),
            ConcreteValue::Big(words) => {
                // base 2 logarithm (rounded down) searches
                // for the highest one and returns its position
                // iterate from the highest word to lowest
                // once the logarithm is obtained, add the bits for lower words
                for i in (0..words.len()).rev() {
                    if let Some(word_ilog2) = words[i].checked_ilog2() {
                        return Some(word_ilog2 + 64 * TryInto::<u32>::try_into(i).unwrap());
                    }
                }
                None
            }
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
            ConcreteValue::Small(value) => {
                assert_eq!(word_index, 0);
                value
            }
            ConcreteValue::Big(words) => &mut words[word_index],
        };

        if set_value {
            *word |= 1 << bit_in_word;
        } else {
            *word &= !(1 << bit_in_word);
        }
    }

    pub fn set_bits(&mut self, lo: u32, hi: u32, set_value: bool) {
        assert!(lo <= hi);
        let lo_word_index: usize = (lo / 64).try_into().unwrap();
        let hi_word_index: usize = (hi / 64).try_into().unwrap();

        let bit_lo = lo % 64;
        let bit_hi = hi % 64;

        let mask_lo = !compute_u64_mask(bit_lo);
        let mask_hi = compute_u64_mask(bit_hi + 1);

        let set_word_value = |word: &mut u64, mask: u64, set_value| {
            *word = if set_value {
                *word | mask
            } else {
                *word & !mask
            }
        };

        match self {
            ConcreteValue::Small(value) => {
                assert_eq!(hi_word_index, 0);
                set_word_value(value, mask_lo & mask_hi, set_value);
            }
            ConcreteValue::Big(words) => {
                if lo_word_index == hi_word_index {
                    set_word_value(&mut words[lo_word_index], mask_lo & mask_hi, set_value);
                } else {
                    // use lo mask in lo word, full mask in words inbetween, hi mask in hi word
                    set_word_value(&mut words[lo_word_index], mask_lo, set_value);
                    for i in lo_word_index + 1..hi_word_index {
                        set_word_value(&mut words[i], u64::MAX, set_value);
                    }
                    set_word_value(&mut words[hi_word_index], mask_hi, set_value);
                }
            }
        }
    }

    pub fn try_to_u32(&self) -> Option<u32> {
        // all words above the lowest must be zero
        if !self.is_zero_above_lowest_word() {
            return None;
        }

        let lowest_word = match self {
            ConcreteValue::Small(small) => *small,
            ConcreteValue::Big(words) => words[0],
        };

        lowest_word.try_into().ok()
    }

    pub fn is_zero_above_lowest_word(&self) -> bool {
        // all words other than zero word must be zero
        match self {
            ConcreteValue::Small(_) => true,
            ConcreteValue::Big(words) => {
                for i in 1..words.len() {
                    if words[i] != 0 {
                        return false;
                    }
                }
                true
            }
        }
    }

    pub fn is_zero(&self) -> bool {
        match &self {
            ConcreteValue::Small(value) => *value == 0,
            ConcreteValue::Big(elems) => elems.iter().all(|e| *e == 0),
        }
    }

    pub fn is_one(&self) -> bool {
        if !self.is_zero_above_lowest_word() {
            return false;
        }

        let first_word = match &self {
            ConcreteValue::Small(value) => *value,
            ConcreteValue::Big(elems) => elems[0],
        };

        first_word == 1
    }
}
