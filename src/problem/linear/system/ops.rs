use vec1::Vec1;

use crate::{
    domain::{
        bitvector::concr::ConcreteBitvector,
        traits::forward::{Bitwise, HwArith},
    },
    problem::linear::{LinearRelation, LinearSystem},
};

impl LinearSystem {
    pub fn bit_not(self) -> Result<Self, bool> {
        // negate universality
        let new_universal = !self.universal;
        let mut new_relations = Vec::new();

        for relation in &self.relations {
            // consider modulus 'm', left side 'a' and right side slack 's'
            // where 0 <= a < m, 0 <= s < m
            // we can now manipulate inequalities without regard to modularity
            // as long as we ensure the end values are within [0, m-1]
            // we want to negate the original inequality !(a <= s) and obtain the same lesser-or-equal form
            // 1. propagate negation into inequality: a > s
            // 2. multiply by minus one: -a < -s
            // 3. add m to both sides: m-a < m-s
            // 4. subtract 1 from right side and change to non-strict inequality: m-a <= m-s-1
            // 5. to bring the left side into bounds, subtract 1 from both sides: m-a-1 <= m-s-2
            // 6. use (!x) = m-x-1 to simplify: (!a) <= (!s)-1
            // for left side, 0 <= (!a) < m, but for right side, -1 <= (!s)-1 < m-1
            // handle the case where (!s) == 0 specially

            let bit_not_slack = relation.slack.bit_not();
            if bit_not_slack.is_zero() {
                // the relation a <= s was a tautology as s was the highest possible value
                // the negated relation will be a contradiction
                if new_universal {
                    // the new system is a conjunction of relations, becomes a contradiction
                    return Err(false);
                }

                // the new system is a disjunction of relations, skip the relation
                continue;
            }

            // we now know 0 <= (!a) < m and 0 <= (!s)-1 < m-1
            // as such, we can construct the relation -a <= (!s-1)
            // as the negation of a <= s

            let combination = relation.combination.clone().bit_not();
            let slack = bit_not_slack.sub(ConcreteBitvector::one(relation.slack.bound()));

            new_relations.push(LinearRelation { combination, slack });
        }

        let Ok(new_relations) = Vec1::try_from_vec(new_relations) else {
            // no relations retained, the system is an empty disjunction of relations
            assert!(!new_universal);
            return Err(true);
        };

        Ok(LinearSystem {
            universal: new_universal,
            relations: new_relations,
        })
    }

    pub fn and(self, rhs: LinearSystem) -> Option<Self> {
        self.combine(rhs, true)
    }

    pub fn or(self, rhs: LinearSystem) -> Option<Self> {
        self.combine(rhs, false)
    }

    fn combine(mut self, rhs: LinearSystem, universal: bool) -> Option<Self> {
        let lhs_compatible = self.relations.len() == 1 || self.universal == universal;
        let rhs_compatible = rhs.relations.len() == 1 || rhs.universal == universal;

        if !lhs_compatible || !rhs_compatible {
            return None;
        }

        self.relations.extend(rhs.relations);
        self.normalize();

        Some(self)
    }
}
