use crate::{
    domain::{
        bitvector::concr::ConcreteBitvector,
        traits::forward::{Bitwise, HwArith},
    },
    problem::operation::linear::{LinearRelation, LinearSystem},
};

impl LinearSystem {
    pub fn bit_not(self) -> Result<Self, bool> {
        let mut new_relations = Vec::new();

        let was_conjuction = match self {
            LinearSystem::Single(_) => None,
            LinearSystem::Disjunction(_) => Some(false),
            LinearSystem::Conjunction(_) => Some(true),
        };

        matches!(self, LinearSystem::Conjunction(_));

        for relation in self.into_relations_iter() {
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

            let bit_not_slack = relation.slack().bit_not();
            if bit_not_slack.is_zero() {
                // the relation a <= s was a tautology as s was the highest possible value

                match was_conjuction {
                    None | Some(false) => {
                        // this was a single tautology or a disjunction with tautology
                        // therefore, it was tautological and the negation is a contradiction
                        return Err(false);
                    }
                    Some(true) => {
                        // this was a conjunction of multiple relations
                        // skip this relation, but still consider the others
                        continue;
                    }
                }
            }

            // we now know 0 <= (!a) < m and 0 <= (!s)-1 < m-1
            // as such, we can construct the relation -a <= (!s-1)
            // as the negation of a <= s

            let combination = relation.combination().clone().bit_not();
            let slack = bit_not_slack.sub(ConcreteBitvector::one(relation.slack().bound()));

            new_relations.push(LinearRelation::new(combination, slack));
        }

        if new_relations.is_empty() {
            // no relations retained
            // this means all relations were tautological and their negation is a contradiction
            return Err(false);
        };

        let mut system = match <[_; 1]>::try_from(new_relations) {
            Ok([new_relation]) => LinearSystem::Single(new_relation),
            Err(new_relations) => match was_conjuction {
                Some(false) => LinearSystem::Conjunction(new_relations),
                Some(true) => LinearSystem::Disjunction(new_relations),
                None => {
                    panic!("Bit-not should not turn a single equation into multiple ones")
                }
            },
        };

        system.normalize();

        Ok(system)
    }

    pub fn and(self, rhs: LinearSystem) -> Option<Self> {
        self.combine(rhs, true)
    }

    pub fn or(self, rhs: LinearSystem) -> Option<Self> {
        self.combine(rhs, false)
    }

    fn combine(self, rhs: LinearSystem, universal: bool) -> Option<Self> {
        let lhs_compatible = matches!(self, LinearSystem::Single(_))
            || universal == matches!(self, LinearSystem::Conjunction(_));
        let rhs_compatible = matches!(rhs, LinearSystem::Single(_))
            || universal == matches!(rhs, LinearSystem::Conjunction(_));

        if !lhs_compatible || !rhs_compatible {
            return None;
        }

        let new_relations: Vec<LinearRelation> = self
            .into_relations_iter()
            .chain(rhs.into_relations_iter())
            .collect();

        assert!(!new_relations.is_empty());

        let mut system = match <[_; 1]>::try_from(new_relations) {
            Ok([new_relation]) => LinearSystem::Single(new_relation),
            Err(new_relations) => {
                if universal {
                    LinearSystem::Conjunction(new_relations)
                } else {
                    LinearSystem::Disjunction(new_relations)
                }
            }
        };

        system.normalize();

        Some(system)
    }
}
