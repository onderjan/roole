use itertools::Itertools;

use crate::domain::{
    bitvector::{
        abstr::{
            BitvectorDomain,
            linear::{LinearBitvector, LinearRelation, LinearStatement, LinearSystem, LinearType},
        },
        concr::ConcreteBitvector,
    },
    traits::forward::{Bitwise, HwArith},
};

impl Bitwise for LinearBitvector {
    fn bit_not(self) -> Self {
        match self.ty {
            LinearType::Top => self,
            LinearType::Combination(combination) => {
                // bit_not(x) = arith_neg(x) - 1

                let mut result = combination.arith_neg();

                result.constant = result.constant.sub(ConcreteBitvector::one(self.bound));

                result.normalize();

                LinearBitvector {
                    bound: self.bound,
                    ty: LinearType::Combination(result),
                }
            }
            LinearType::System(linear_system) => {
                let Ok(statement) = linear_system.equations.into_iter().exactly_one() else {
                    return Self::top(self.bound);
                };

                LinearBitvector {
                    bound: self.bound,
                    ty: LinearType::System(super::LinearSystem {
                        equations: vec![statement.negate()],
                    }),
                }
            }
        }
    }
    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let bound = self.bound;

        match (self.ty, rhs.ty) {
            (LinearType::System(lhs), LinearType::System(mut rhs)) => {
                let mut equations = lhs.equations;
                equations.append(&mut rhs.equations);
                Self {
                    bound,
                    ty: LinearType::System(LinearSystem { equations }),
                }
            }
            _ => Self::top(bound),
        }
    }
    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        // TODO: handle masking situations

        LinearBitvector::top(self.bound)
    }
    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        // TODO: handle masking situations

        LinearBitvector::top(self.bound)
    }
}

impl LinearStatement {
    fn negate(self) -> Self {
        let op = match self.op {
            LinearRelation::Eq => LinearRelation::Ne,
            LinearRelation::Ne => LinearRelation::Eq,
            super::LinearRelation::Lt => todo!("Negate inequality"),
        };

        Self {
            left: self.left,
            op,
            right: self.right,
        }
    }
}
