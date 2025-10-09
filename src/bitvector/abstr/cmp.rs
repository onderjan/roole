use crate::bitvector::abstr::Primitive;

use super::ThreeValued;

impl<T: Primitive> ThreeValued<T> {
    fn ult(self, rhs: Self, width: u32) -> Self {
        // use unsigned versions
        let lhs_min = self.umin(width);
        let lhs_max = self.umax(width);
        let rhs_min = rhs.umin(width);
        let rhs_max = rhs.umax(width);

        // can be zero if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_zero = lhs_max >= rhs_min;

        // can be one if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_one = lhs_min < rhs_max;

        ThreeValued::from_bools(result_can_be_zero, result_can_be_one)
    }

    fn ule(self, rhs: Self, width: u32) -> Self {
        // use unsigned versions
        let lhs_min = self.umin(width);
        let lhs_max = self.umax(width);
        let rhs_min = rhs.umin(width);
        let rhs_max = rhs.umax(width);

        // can be zero if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_zero = lhs_max > rhs_min;

        // can be one if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_one = lhs_min <= rhs_max;

        ThreeValued::from_bools(result_can_be_zero, result_can_be_one)
    }

    fn slt(self, rhs: Self, width: u32) -> Self {
        // use signed versions
        let lhs_min = self.smin(width);
        let lhs_max = self.smax(width);
        let rhs_min = rhs.smin(width);
        let rhs_max = rhs.smax(width);

        // can be zero if lhs can be greater or equal to rhs
        // this is only possible if lhs max can be greater or equal to rhs min
        let result_can_be_zero = lhs_max >= rhs_min;

        // can be one if lhs can be lesser than rhs
        // this is only possible if lhs min can be lesser than rhs max
        let result_can_be_one = lhs_min < rhs_max;

        ThreeValued::from_bools(result_can_be_zero, result_can_be_one)
    }

    fn sle(self, rhs: Self, width: u32) -> Self {
        // use signed versions
        let lhs_min = self.smin(width);
        let lhs_max = self.smax(width);
        let rhs_min = rhs.smin(width);
        let rhs_max = rhs.smax(width);

        // can be zero if lhs can be greater than rhs
        // this is only possible if lhs max can be greater to rhs min
        let result_can_be_zero = lhs_max > rhs_min;

        // can be one if lhs can be lesser or equal to rhs
        // this is only possible if lhs min can be lesser or equal to rhs max
        let result_can_be_one = lhs_min <= rhs_max;

        ThreeValued::from_bools(result_can_be_zero, result_can_be_one)
    }
}
