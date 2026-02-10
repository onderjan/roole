use std::collections::BTreeMap;
use std::fmt::{Debug, UpperHex};

use super::{LinearMonomial, LinearPolynomial, LinearSlice};
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::formula::FormulaId,
};

impl LinearPolynomial {
    pub fn new(
        linear_terms: BTreeMap<LinearSlice, ConcreteBitvector<RBound>>,
        constant_term: ConcreteBitvector<RBound>,
    ) -> Self {
        let result = Self {
            constant_term,
            linear_terms,
        };
        result.into_normal_form()
    }

    pub fn empty(bound: RBound) -> Self {
        Self {
            linear_terms: BTreeMap::new(),
            constant_term: ConcreteBitvector::zero(bound),
        }
    }

    pub fn from_monomial_and_constant(
        monomial: LinearMonomial,
        constant_term: ConcreteBitvector<RBound>,
    ) -> Self {
        Self::new(
            BTreeMap::from_iter([(monomial.slice, monomial.coefficient)]),
            constant_term,
        )
    }

    pub fn from_monomial(monomial: LinearMonomial) -> Self {
        Self::new(
            BTreeMap::from_iter([(monomial.slice, monomial.coefficient)]),
            ConcreteBitvector::zero(monomial.bound()),
        )
    }

    pub fn from_constant(constant: ConcreteBitvector<RBound>) -> Self {
        Self {
            linear_terms: BTreeMap::new(),
            constant_term: constant,
        }
    }

    pub fn from_formula(formula_id: FormulaId, bound: RBound) -> Self {
        if let Some(slice) = LinearSlice::from_bounded(formula_id, bound) {
            let coefficient = ConcreteBitvector::one(bound);
            LinearPolynomial::from_monomial(LinearMonomial::new(coefficient, slice))
        } else {
            LinearPolynomial::empty(bound)
        }
    }

    pub fn bound(&self) -> RBound {
        self.constant_term.bound()
    }

    pub fn single_bit(constant: bool) -> LinearPolynomial {
        let bound = RBound::single_bit_bound();
        let constant = if constant {
            ConcreteBitvector::one(bound)
        } else {
            ConcreteBitvector::zero(bound)
        };

        LinearPolynomial::from_constant(constant)
    }

    pub(crate) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
        if self.bound().width() == 1 && self.linear_terms.len() == 1 {
            // simplify printing Boolean polynomials with a single term
            let Some((slice, coefficient)) = self.linear_terms.iter().next() else {
                panic!("There should be a linear term");
            };

            if slice.width.get() == 1 && coefficient.is_one() {
                // only a single linear term with single-bit slice and coefficient one
                // just print the slice, negated if the constant term is nonzero (i.e. one)
                if self.constant_term.is_nonzero() {
                    write!(f, "!")?;
                }
                return write!(f, "{:?}", slice);
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
        for (slice, coefficient) in &self.linear_terms {
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
                coefficient.arith_neg()
            } else {
                *coefficient
            };
            if !abs_coefficient.is_one() {
                if hex {
                    write!(f, "{:#X}*", abs_coefficient)?;
                } else {
                    write!(f, "{:?}*", abs_coefficient)?;
                }
            }

            write!(f, "{:?}", slice)?;
        }

        let abs_constant_term = if self.constant_term.is_nonzero() {
            let write_as_negative =
                !hex && (self.constant_term.is_sign_bit_set() && !self.constant_term.is_overhalf());
            let abs_constant_term = if write_as_negative {
                self.constant_term.arith_neg()
            } else {
                self.constant_term
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
            Some(self.constant_term)
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
