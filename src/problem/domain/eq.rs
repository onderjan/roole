use vec1::vec1;

use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::forward::{Bitwise, TypedEq},
    },
    problem::{
        domain::OperationDomain,
        linear::{LinearRelation, LinearSystem},
    },
};

impl TypedEq for OperationDomain {
    type Output = OperationDomain;
    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound(), rhs.bound());

        let (OperationDomain::Combination(lhs), OperationDomain::Combination(rhs)) = (self, rhs)
        else {
            return OperationDomain::top(RBound::single_bit_bound());
        };

        // if both are combinations, make into an equality

        let combination = lhs.sub(rhs);
        let slack = ConcreteBitvector::zero(combination.constant.bound());

        let system = LinearSystem {
            universal: true,
            relations: vec1![LinearRelation { combination, slack }],
        };
        OperationDomain::System(system)
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}
