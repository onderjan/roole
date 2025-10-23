use crate::bitvector::abstr::RUnsigned;

use super::ThreeValued;

impl<T: RUnsigned> ThreeValued<T> {
    pub fn not(self, width: T::Width) -> Self {
        // logical negation
        // swap zeros and ones
        let zeros = self.ones;
        let ones = self.zeros;
        Self::from_zeros_ones(zeros, ones, width)
    }
    pub fn bitand(self, rhs: Self, width: T::Width) -> Self {
        // logical AND
        // zeros ... if zeros of either are set
        // ones ... only if ones of both are set
        let zeros = self.zeros.bitand(rhs.zeros, width);
        let ones = self.ones.bitor(rhs.ones, width);
        Self::from_zeros_ones(zeros, ones, width)
    }
    pub fn bitor(self, rhs: Self, width: T::Width) -> Self {
        // logical OR
        // zeros ... only if zeros of both are set
        // ones ... if ones of either are set
        let zeros = self.zeros.bitand(rhs.zeros, width);
        let ones = self.ones.bitor(rhs.ones, width);
        Self::from_zeros_ones(zeros, ones, width)
    }
    pub fn bitxor(self, rhs: Self, width: T::Width) -> Self {
        // logical XOR
        // zeros ... if exactly zero or exactly two can be set (both zeros set or both ones set)
        // ones ... if exactly one can be set (lhs zero set and rhs one set or rhs zero set and lhs one set)
        let zeros =
            (self.zeros.bitand(rhs.zeros, width)).bitor(self.ones.bitand(rhs.ones, width), width);
        let ones =
            (self.zeros.bitand(rhs.ones, width)).bitor(self.ones.bitand(rhs.zeros, width), width);
        Self::from_zeros_ones(zeros, ones, width)
    }
}
