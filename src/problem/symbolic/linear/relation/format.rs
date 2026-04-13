use std::fmt::{Debug, UpperHex};

use crate::domain::{bitvector::concr::ConcreteBitvector, traits::forward::HwArith};

use super::{super::LinearPolynomial, LinearRelation};

impl LinearRelation {
    pub(crate) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        let one = ConcreteBitvector::new_one(self.polynomial.bound());
        if self.slack.clone().add(one.clone()).is_full_mask() {
            // better to add 1 to the polynomial and print as non-equality
            let nonequality_polynomial = self
                .polynomial
                .clone()
                .add(LinearPolynomial::from_concrete(one));

            if hex {
                UpperHex::fmt(&nonequality_polynomial, f)?;
            } else {
                Debug::fmt(&nonequality_polynomial, f)?;
            }

            write!(f, " != 0")
        } else {
            if hex {
                UpperHex::fmt(&self.polynomial, f)?;
            } else {
                Debug::fmt(&self.polynomial, f)?;
            }

            let op = if self.slack.is_zero() { "==" } else { "<=" };

            write!(f, " {} ", op)?;

            if hex {
                write!(f, "{:#X}", self.slack)
            } else {
                write!(f, "{:?}", self.slack)
            }
        }
    }
}

impl Debug for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for LinearRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
