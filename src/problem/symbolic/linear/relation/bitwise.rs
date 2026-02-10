use crate::domain::{
    bitvector::concr::ConcreteBitvector,
    traits::forward::{Bitwise, HwArith},
};

use super::{super::LinearPolynomial, LinearRelation};

impl LinearRelation {
    pub fn bit_not(self) -> Result<Self, LinearPolynomial> {
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

        let bit_not_slack = self.slack().bit_not();
        if bit_not_slack.is_zero() {
            // the relation a <= s was a tautology as s was the highest possible value
            // return contradiction
            return Err(LinearPolynomial::from_bool(false));
        }

        // we now know 0 <= (!a) < m and 0 <= (!s)-1 < m-1
        // as such, we can construct the relation -a <= (!s-1)
        // as the negation of a <= s

        let polynomial = self.polynomial.clone().bit_not();
        let slack = bit_not_slack.sub(ConcreteBitvector::one(self.slack.bound()));

        Ok(LinearRelation::new(polynomial, slack))
    }
}
