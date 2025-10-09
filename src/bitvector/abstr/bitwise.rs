use crate::bitvector::abstr::Primitive;

use super::ThreeValued;

impl<T: Primitive> ThreeValued<T> {
    pub fn bit_not(self, width: u32) -> Self {
        // logical negation
        // swap zeros and ones
        let zeros = self.ones;
        let ones = self.zeros;
        Self::from_zeros_ones(zeros, ones, width)
    }
    pub fn bit_and(self, rhs: Self, width: u32) -> Self {
        // logical AND
        // zeros ... if zeros of either are set
        // ones ... only if ones of both are set
        let zeros = self.zeros | rhs.zeros;
        let ones = self.ones & rhs.ones;
        Self::from_zeros_ones(zeros, ones, width)
    }
    pub fn bit_or(self, rhs: Self, width: u32) -> Self {
        // logical OR
        // zeros ... only if zeros of both are set
        // ones ... if ones of either are set
        let zeros = self.zeros & rhs.zeros;
        let ones = self.ones | rhs.ones;
        Self::from_zeros_ones(zeros, ones, width)
    }
    pub fn bit_xor(self, rhs: Self, width: u32) -> Self {
        // logical XOR
        // zeros ... if exactly zero or exactly two can be set (both zeros set or both ones set)
        // ones ... if exactly one can be set (lhs zero set and rhs one set or rhs zero set and lhs one set)
        let zeros = (self.zeros & rhs.zeros) | (self.ones & rhs.ones);
        let ones = (self.zeros & rhs.ones) | (self.ones & rhs.zeros);
        Self::from_zeros_ones(zeros, ones, width)
    }
}
