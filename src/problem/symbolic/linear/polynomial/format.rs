use std::fmt::{Debug, UpperHex};

use super::LinearPolynomial;
use crate::domain::{bitvector::BitvectorBound, traits::forward::HwArith};

impl LinearPolynomial {
    pub(crate) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        //write!(f, "({:?}/{:?})", self.linear_terms, self.constant_term)?;
        if self.bound().width() == 1 && self.linear_terms.len() == 1 {
            // simplify printing Boolean polynomials with a single term
            let Some(monomial) = self.linear_terms.first() else {
                panic!("There should be a linear term");
            };

            if monomial.slice.width.get() == 1 && monomial.coefficient.is_one() {
                // only a single linear term with single-bit slice and coefficient one
                // just print the slice, negated if the constant term is nonzero (i.e. one)
                if self.constant_term.is_nonzero() {
                    write!(f, "!")?;
                }
                return write!(f, "{:?}", monomial.slice);
            }
        }

        let mut is_first = true;

        let num_linear_terms = self.linear_terms.len();
        let write_parentheses =
            num_linear_terms > 1 || num_linear_terms == 1 && self.constant_term.is_nonzero();

        if write_parentheses {
            write!(f, "(")?;
        }

        // write the linear monomials
        for monomial in &self.linear_terms {
            let coefficient = &monomial.coefficient;

            let write_as_negative =
                !hex && (coefficient.is_sign_bit_set() && !coefficient.is_overhalf());

            if is_first {
                if write_as_negative {
                    write!(f, "-")?;
                }
                is_first = false;
            } else if write_as_negative {
                write!(f, " - ")?;
            } else {
                write!(f, " + ")?;
            }

            let abs_coefficient = if write_as_negative {
                coefficient.clone().arith_neg()
            } else {
                coefficient.clone()
            };
            if !abs_coefficient.is_one() {
                if hex {
                    write!(f, "{:#X}*", abs_coefficient)?;
                } else {
                    write!(f, "{:?}*", abs_coefficient)?;
                }
            }

            write!(f, "{:?}", monomial.slice)?;
        }

        let abs_constant_term = if self.constant_term.is_nonzero() {
            let write_as_negative =
                !hex && (self.constant_term.is_sign_bit_set() && !self.constant_term.is_overhalf());
            let abs_constant_term = if write_as_negative {
                self.constant_term.clone().arith_neg()
            } else {
                self.constant_term.clone()
            };

            match (is_first, write_as_negative) {
                (false, false) => {
                    write!(f, " + ")?;
                }
                (false, true) => {
                    write!(f, " - ")?;
                }
                (true, false) => {}
                (true, true) => {
                    write!(f, "-")?;
                }
            }

            Some(abs_constant_term)
        } else if is_first {
            Some(self.constant_term.clone())
        } else {
            None
        };
        if let Some(abs_constant_term) = abs_constant_term {
            if hex {
                write!(f, "{:#X}", abs_constant_term)?;
            } else {
                write!(f, "{:?}", abs_constant_term)?;
            }
        }

        if write_parentheses {
            write!(f, ")")?;
        }

        write!(f, " mod ")?;
        if hex {
            write!(f, "{:#X}", 1u128 << self.bound().width())
        } else {
            write!(f, "{:?}", 1u128 << self.bound().width())
        }
    }
}

impl Debug for LinearPolynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, false)
    }
}

impl UpperHex for LinearPolynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, true)
    }
}
