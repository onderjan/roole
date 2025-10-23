use std::fmt::Debug;

pub trait RUnsigned: Clone + Copy + PartialEq + Eq + Debug {
    type Width: Clone + Copy + Debug;
    type Index: Clone + Copy + Debug;

    fn add(self, rhs: Self, width: Self::Width) -> Self;
    fn sub(self, rhs: Self, width: Self::Width) -> Self;

    fn not(self, width: Self::Width) -> Self;
    fn bitand(self, rhs: Self, width: Self::Width) -> Self;
    fn bitor(self, rhs: Self, width: Self::Width) -> Self;
    fn bitxor(self, rhs: Self, width: Self::Width) -> Self;

    fn eq(self, rhs: Self, width: Self::Width) -> Self;

    fn zero(width: Self::Width) -> Self;
    fn max_value(width: Self::Width) -> Self;

    fn limited(self, width: Self::Width) -> Self;

    fn single_bit_width() -> Self::Width;
    fn width_up_to(index: Self::Index) -> Self::Width;

    fn index_iter(width: Self::Width) -> impl Iterator<Item = Self::Index>;
    fn index_flag(index: Self::Index) -> Self;

    fn add_shr(self, rhs: Self, least_index: Self::Index, width: Self::Width) -> Self;
    fn sub_shr(self, rhs: Self, least_index: Self::Index, width: Self::Width) -> Self;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RUnsignedU64(pub u64);

impl RUnsigned for RUnsignedU64 {
    type Width = u32;
    type Index = u32;

    fn add(self, rhs: Self, width: Self::Width) -> Self {
        Self((self.0 + rhs.0) & Self::width_mask(width))
    }

    fn sub(self, rhs: Self, width: Self::Width) -> Self {
        Self((self.0 - rhs.0) & Self::width_mask(width))
    }

    fn not(self, width: Self::Width) -> Self {
        Self((!self.0) & Self::width_mask(width))
    }

    fn bitand(self, rhs: Self, _width: Self::Width) -> Self {
        // no masking necessary if both fit
        Self(self.0 & rhs.0)
    }

    fn bitor(self, rhs: Self, _width: Self::Width) -> Self {
        // no masking necessary if both fit
        Self(self.0 | rhs.0)
    }

    fn bitxor(self, rhs: Self, width: Self::Width) -> Self {
        Self((self.0 ^ rhs.0) & Self::width_mask(width))
    }

    fn eq(self, rhs: Self, _width: Self::Width) -> Self {
        // no masking necessary if both fit
        Self((self.0 == rhs.0) as u64)
    }

    fn zero(_width: Self::Width) -> Self {
        Self(0)
    }

    fn max_value(width: Self::Width) -> Self {
        Self(Self::width_mask(width))
    }

    fn limited(self, width: Self::Width) -> Self {
        Self(self.0 & Self::width_mask(width))
    }

    fn single_bit_width() -> Self::Width {
        1
    }

    fn index_iter(width: Self::Width) -> impl Iterator<Item = Self::Index> {
        0..width
    }

    fn index_flag(index: Self::Index) -> Self {
        Self(1 << index)
    }

    fn width_up_to(index: Self::Index) -> Self::Width {
        index + 1
    }

    fn add_shr(self, rhs: Self, least_index: Self::Index, width: Self::Width) -> Self {
        Self(shr_overflowing(self.0.overflowing_add(rhs.0), least_index) & Self::width_mask(width))
    }

    fn sub_shr(self, rhs: Self, least_index: Self::Index, width: Self::Width) -> Self {
        Self(shr_overflowing(self.0.overflowing_sub(rhs.0), least_index) & Self::width_mask(width))
    }
}

fn shr_overflowing(overflowing_result: (u64, bool), k: u32) -> u64 {
    let mut result = overflowing_result.0 >> k;
    if overflowing_result.1 && k > 0 {
        let overflow_pos = u64::BITS - k;
        result |= 1u64 << overflow_pos;
    }
    result
}

impl RUnsignedU64 {
    fn width_mask(width: <Self as RUnsigned>::Width) -> u64 {
        if width == 0 {
            return 0;
        }
        if width == u64::BITS {
            // this would fail in checked shl,
            // but the mask is just full of ones
            return 0u64.wrapping_sub(1u64);
        }
        let num_values = u64::checked_shl(1u64, width);
        let Some(num_values) = num_values else {
            panic!("Bit mask length should fit");
        };

        num_values.wrapping_sub(1u64)
    }
}
