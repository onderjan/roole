use crate::domain::{
    bitvector::{
        BitvectorBound, RBound,
        abstr::{
            BitvectorDomain,
            linear::{LinearBitvector, LinearEquation, LinearSystem, LinearType},
        },
    },
    traits::forward::TypedEq,
};

impl TypedEq for LinearBitvector {
    type Output = LinearBitvector;
    fn eq(self, rhs: Self) -> Self::Output {
        assert_eq!(self.bound, rhs.bound);

        let (LinearType::Combination(lhs), LinearType::Combination(rhs)) = (self.ty, rhs.ty) else {
            return LinearBitvector::top(RBound::single_bit_bound());
        };

        // if both are combinations, make into an equation

        let side = lhs.sub(rhs);

        let system = LinearSystem {
            equations: vec![LinearEquation { side }],
        };
        Self::Output {
            bound: RBound::single_bit_bound(),
            ty: LinearType::System(system),
        }
    }

    fn ne(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
