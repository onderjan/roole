use std::collections::BTreeMap;

use itertools::Itertools;

use crate::domain::{
    bitvector::{
        abstr::{
            BitvectorDomain,
            linear::{
                LinearBitvector, LinearCombination, LinearRelation, LinearSystem, LinearType,
            },
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
                let Ok(relation) = linear_system.relations.into_iter().exactly_one() else {
                    return Self::top(self.bound);
                };

                let result = match relation {
                    LinearRelation::Eq(combination) => LinearRelation::Ne(combination),
                    LinearRelation::Ne(combination) => LinearRelation::Eq(combination),
                };

                LinearBitvector {
                    bound: self.bound,
                    ty: LinearType::System(super::LinearSystem {
                        relations: vec![result],
                    }),
                }
            }
        }
    }

    fn bit_and(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let bound = self.bound;

        match (self.ty, rhs.ty) {
            (LinearType::System(lhs), LinearType::System(rhs)) => {
                let mut relations = lhs.relations;
                let num_lhs_relations = relations.len();

                // remove duplicates
                for rhs_relation in rhs.relations {
                    let mut unnecessary = false;

                    for lhs_relation in relations.iter().take(num_lhs_relations) {
                        match (lhs_relation, &rhs_relation) {
                            (
                                LinearRelation::Eq(lhs_combination),
                                LinearRelation::Eq(rhs_combination),
                            )
                            | (
                                LinearRelation::Ne(lhs_combination),
                                LinearRelation::Ne(rhs_combination),
                            ) => {
                                if lhs_combination == rhs_combination {
                                    unnecessary = true;
                                    break;
                                }
                            }
                            (
                                LinearRelation::Eq(lhs_combination),
                                LinearRelation::Ne(rhs_combination),
                            )
                            | (
                                LinearRelation::Ne(lhs_combination),
                                LinearRelation::Eq(rhs_combination),
                            ) => {
                                if lhs_combination == rhs_combination {
                                    // opposing equations
                                    return LinearBitvector {
                                        bound,
                                        ty: LinearType::Combination(LinearCombination {
                                            constant: ConcreteBitvector::zero(bound),
                                            coefficients: BTreeMap::new(),
                                        }),
                                    };
                                }
                            }
                        }
                    }

                    if !unnecessary {
                        relations.push(rhs_relation);
                    }
                }

                Self {
                    bound,
                    ty: LinearType::System(LinearSystem { relations }),
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
