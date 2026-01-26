use vec1::vec1;

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::forward::{Bitwise, TypedEq},
    },
    problem::domain::{LinearBitvector, LinearRelation, LinearSystem},
};

impl TypedEq for LinearBitvector {
    type Output = LinearBitvector;
    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound(), rhs.bound());

        let (LinearBitvector::Combination(lhs), LinearBitvector::Combination(rhs)) = (self, rhs)
        else {
            return LinearBitvector::top(RBound::single_bit_bound());
        };

        // if both are combinations, make into an equality

        let combination = lhs.sub(rhs);
        let slack = ConcreteBitvector::zero(combination.constant.bound());

        let system = LinearSystem {
            universal: true,
            relations: vec1![LinearRelation { combination, slack }],
        };
        LinearBitvector::System(system)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}
