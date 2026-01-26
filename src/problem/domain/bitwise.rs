use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, abstr::BitvectorDomain, concr::ConcreteBitvector},
        traits::forward::{Bitwise, HwArith},
    },
    problem::domain::{LinearBitvector, LinearSystem},
};

impl Bitwise for LinearBitvector {
    fn bit_not(self) -> Self {
        match self {
            LinearBitvector::Top(_) => self,
            LinearBitvector::Combination(combination) => {
                // bit_not(x) = arith_neg(x) - 1

                let mut result = combination.arith_neg();

                result.constant = result.constant.sub(ConcreteBitvector::one(result.bound()));

                result.normalize();

                LinearBitvector::Combination(result)
            }
            LinearBitvector::System(mut system) => {
                // negate universality
                system.universal = !system.universal;

                for relation in &mut system.relations {
                    // to properly negate, start after the end of the old wrapping interval
                    let old_length = relation
                        .slack
                        .add(ConcreteBitvector::one(relation.slack.bound()));
                    relation.combination.constant = relation.combination.constant.add(old_length);

                    // old wrapping interval length is slack + 1
                    // e.g. a + [0,3] == 0 mod 16 has interval length 4
                    // we want the new interval length equal to modulus - old length, i.e. modulus - old_slack - 1
                    // the new slack is length - 1, i.e. modulus - old_slack - 2
                    // since !x = -x-1, this can be also represented as (!old_slack) - 1
                    relation.slack = relation
                        .slack
                        .bit_not()
                        .sub(ConcreteBitvector::one(relation.slack.bound()));
                }

                LinearBitvector::System(system)
            }
        }
    }

    fn bit_and(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        match (self, rhs) {
            (LinearBitvector::System(lhs), LinearBitvector::System(rhs)) => {
                merge_systems(lhs, rhs, true)
            }
            _ => Self::top(bound),
        }
    }
    fn bit_or(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        match (self, rhs) {
            (LinearBitvector::System(lhs), LinearBitvector::System(rhs)) => {
                merge_systems(lhs, rhs, false)
            }
            _ => Self::top(bound),
        }
    }
    fn bit_xor(self, rhs: Self) -> Self {
        let bound = self.bound();
        assert_eq!(bound, rhs.bound());
        // TODO: handle masking situations

        LinearBitvector::top(bound)
    }
}

fn merge_systems(lhs: LinearSystem, rhs: LinearSystem, universal: bool) -> LinearBitvector {
    let lhs_compatible = lhs.relations.len() == 1 || lhs.universal == universal;
    let rhs_compatible = rhs.relations.len() == 1 || rhs.universal == universal;

    if !lhs_compatible || !rhs_compatible {
        return LinearBitvector::Top(RBound::single_bit_bound());
    }

    let mut system = lhs;
    system.relations.extend(rhs.relations);
    system.normalize();

    LinearBitvector::System(system)
}
