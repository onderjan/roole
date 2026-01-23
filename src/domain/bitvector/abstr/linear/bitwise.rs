use std::collections::BTreeMap;

use crate::domain::{
    bitvector::{
        RBound,
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
            LinearType::System(system) => {
                let mut relations = Vec::new();
                for relation in system.relations {
                    relations.push(match relation {
                        LinearRelation::Eq(combination) => LinearRelation::Ne(combination),
                        LinearRelation::Ne(combination) => LinearRelation::Eq(combination),
                    });
                }

                LinearBitvector {
                    bound: self.bound,
                    ty: LinearType::System(LinearSystem {
                        universal: !system.universal,
                        relations,
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
                merge_systems(lhs, rhs, bound, true)
            }
            _ => Self::top(bound),
        }
    }
    fn bit_or(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        let bound = self.bound;

        match (self.ty, rhs.ty) {
            (LinearType::System(lhs), LinearType::System(rhs)) => {
                merge_systems(lhs, rhs, bound, false)
            }
            _ => Self::top(bound),
        }
    }
    fn bit_xor(self, rhs: Self) -> Self {
        assert_eq!(self.bound, rhs.bound);
        // TODO: handle masking situations

        LinearBitvector::top(self.bound)
    }
}

fn merge_systems(
    lhs: LinearSystem,
    rhs: LinearSystem,
    bound: RBound,
    universal: bool,
) -> LinearBitvector {
    let lhs_compatible = lhs.relations.len() == 1 || lhs.universal == universal;
    let rhs_compatible = rhs.relations.len() == 1 || rhs.universal == universal;

    if !lhs_compatible || !rhs_compatible {
        return LinearBitvector {
            bound,
            ty: LinearType::Top,
        };
    }

    let mut relations = lhs.relations;
    let num_lhs_relations = relations.len();

    // remove duplicates
    for rhs_relation in rhs.relations {
        let mut unnecessary = false;

        for lhs_relation in relations.iter().take(num_lhs_relations) {
            match (lhs_relation, &rhs_relation) {
                (LinearRelation::Eq(lhs_combination), LinearRelation::Eq(rhs_combination))
                | (LinearRelation::Ne(lhs_combination), LinearRelation::Ne(rhs_combination)) => {
                    if lhs_combination == rhs_combination {
                        unnecessary = true;
                        break;
                    }
                }
                (LinearRelation::Eq(lhs_combination), LinearRelation::Ne(rhs_combination))
                | (LinearRelation::Ne(lhs_combination), LinearRelation::Eq(rhs_combination)) => {
                    if lhs_combination == rhs_combination {
                        // opposing equations
                        let constant = if universal {
                            ConcreteBitvector::zero(bound)
                        } else {
                            ConcreteBitvector::one(bound)
                        };

                        return LinearBitvector {
                            bound,
                            ty: LinearType::Combination(LinearCombination {
                                constant,
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

    LinearBitvector {
        bound,
        ty: LinearType::System(LinearSystem {
            universal,
            relations,
        }),
    }
}
