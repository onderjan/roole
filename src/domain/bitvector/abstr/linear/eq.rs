use crate::domain::{
    bitvector::{
        BitvectorBound, RBound,
        abstr::{
            BitvectorDomain,
            linear::{LinearBitvector, LinearRelation, LinearSystem, LinearType},
        },
    },
    traits::forward::{Bitwise, TypedEq},
};

impl TypedEq for LinearBitvector {
    type Output = LinearBitvector;
    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);

        let (LinearType::Combination(lhs), LinearType::Combination(rhs)) = (self.ty, rhs.ty) else {
            return LinearBitvector::top(RBound::single_bit_bound());
        };

        // if both are combinations, make into an equality

        let combination = lhs.sub(rhs);

        let system = LinearSystem {
            universal: true,
            relations: vec![LinearRelation::Eq(combination)],
        };
        Self::Output {
            bound: RBound::single_bit_bound(),
            ty: LinearType::System(system),
        }
    }

    fn ne(self, rhs: Self) -> Self::Output {
        self.eq(rhs).bit_not()
    }
}
