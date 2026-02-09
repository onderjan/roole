use std::collections::BTreeMap;
use std::fmt::{Debug, UpperHex};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::problem::operation::linear::monomial::LinearMonomial;
use crate::problem::operation::linear::slice::LinearSlice;
use crate::{
    domain::{
        bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
        traits::forward::HwArith,
    },
    problem::{eval::EvaluableDomain, formula::FormulaId},
};

mod arith;
mod bitwise;
mod eq;
mod ext;
mod shift;

/// A linear combination of bitvectors and a constant.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearPolynomial {
    linear_terms: BTreeMap<LinearSlice, ConcreteBitvector<RBound>>,
    constant_term: ConcreteBitvector<RBound>,
}

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

    pub fn used_ids(&self) -> Vec<FormulaId> {
        self.linear_terms
            .keys()
            .map(|slice| slice.formula_id)
            .collect()
    }

    pub fn bound(&self) -> RBound {
        self.constant_term.bound()
    }

    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        let mut value = D::single_value(self.constant_term);
        let polynomial_bound = value.bound();

        for (slice, coefficient) in &self.linear_terms {
            let mut formula_value = (fetch)(slice.formula_id);
            let bound = formula_value.bound();
            // slice
            // first, unsigned shift right to lsb if nonzero
            if slice.lsb != 0 {
                let lsb = ConcreteBitvector::new(slice.lsb.into(), bound);
                formula_value = formula_value.logic_shr(D::single_value(lsb));
            }

            let slice_bound = RBound::new(slice.width.get());

            // perform unsigned extension to slice width unless it is a no-op
            if formula_value.bound() != slice_bound {
                formula_value = formula_value.uext(slice_bound);
            }

            // unless the slice width is equal to polynomial width,
            // perform unsigned extension to it
            if slice_bound != polynomial_bound {
                formula_value = formula_value.uext(polynomial_bound);
            }

            // then, multiply by the coefficient
            let term_value = formula_value.mul(D::single_value(*coefficient));
            value = value.add(term_value);
        }

        value
    }

    pub(super) fn into_normal_form(mut self) -> Self {
        // eliminate zero coefficients
        self.linear_terms.retain(|_, coeff| !coeff.is_zero());
        self
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        let remap = |slice: LinearSlice| {
            let Some(new_id) = old_to_new.get(&slice.formula_id) else {
                panic!(
                    "Used formula id {:?} should be remappable",
                    slice.formula_id
                );
            };
            LinearSlice {
                formula_id: *new_id,
                lsb: slice.lsb,
                width: slice.width,
            }
        };

        let mut old_monomials = BTreeMap::new();
        std::mem::swap(&mut self.linear_terms, &mut old_monomials);

        for (formula_id, coefficient) in old_monomials {
            self.linear_terms.insert(remap(formula_id), coefficient);
        }
    }

    pub fn scale(&mut self, scaler: ConcreteBitvector<RBound>) {
        let bound = self.bound();
        assert_eq!(bound, scaler.bound());

        self.constant_term = self.constant_term.mul(scaler);

        for coefficient in self.linear_terms.values_mut() {
            *coefficient = coefficient.mul(scaler);
        }
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

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        if self.linear_terms.is_empty() {
            Some(self.constant_term)
        } else {
            None
        }
    }

    pub fn constant_value_with_assumption(
        &self,
        assumption: &LinearPolynomial,
    ) -> Option<ConcreteBitvector<RBound>> {
        if let Some(constant_value) = self.constant_value() {
            return Some(constant_value);
        }

        if assumption.bound().width() != 1 {
            return None;
        }

        let Ok((assumption_slice, assumption_factor)) =
            assumption.linear_terms.iter().exactly_one()
        else {
            return None;
        };

        if assumption_slice.width.get() != 1 || !assumption_factor.is_one() {
            return None;
        }

        let Ok((our_slice, our_factor)) = self.linear_terms.iter().exactly_one() else {
            return None;
        };

        if our_slice != assumption_slice {
            return None;
        }

        let mut result = self.constant_term;

        let slice_holds = assumption.constant_term.is_zero();

        if slice_holds {
            result = result.add(*our_factor);
        }

        Some(result)
    }

    pub fn monomial_and_constant_value(
        &self,
    ) -> Option<(Option<LinearMonomial>, ConcreteBitvector<RBound>)> {
        if self.linear_terms.is_empty() {
            return Some((None, self.constant_term));
        }
        let Ok((slice, coefficient)) = self.linear_terms.iter().exactly_one() else {
            return None;
        };

        Some((
            Some(LinearMonomial::new(*coefficient, *slice)),
            self.constant_term,
        ))
    }

    pub fn might_overflow(&self) -> bool {
        if self.linear_terms.is_empty() {
            // only constant, definitely cannot overflow
            return false;
        }

        // TODO: determine if the polynomial might overflow more finely

        let Some((monomial, constant)) = self.monomial_and_constant_value() else {
            // we are unsure, return true
            return true;
        };

        let Some(monomial) = monomial else {
            // just a constant, definitely cannot overflow
            return false;
        };

        if constant.is_nonzero() {
            // we are unsure, return true
            return true;
        }

        monomial.might_overflow()
    }

    pub(super) fn format(&self, f: &mut std::fmt::Formatter<'_>, hex: bool) -> std::fmt::Result {
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

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, num::NonZero};

    use crate::{
        domain::bitvector::{RBound, concr::ConcreteBitvector},
        problem::formula::{FormulaId, VariableId},
    };

    use super::*;

    #[test]
    fn test_addsub() {
        let bound = RBound::new(32);
        let slice = LinearSlice {
            formula_id: FormulaId::Variable(VariableId(0)),
            lsb: 0,
            width: NonZero::new(32).unwrap(),
        };
        let a = LinearPolynomial {
            constant_term: ConcreteBitvector::new(38, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(12, bound))]),
        };
        let b = LinearPolynomial {
            constant_term: ConcreteBitvector::new(17, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(7, bound))]),
        };
        let add_result = LinearPolynomial {
            constant_term: ConcreteBitvector::new(55, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(19, bound))]),
        };
        let sub_result = LinearPolynomial {
            constant_term: ConcreteBitvector::new(21, bound),
            linear_terms: BTreeMap::from_iter([(slice, ConcreteBitvector::new(5, bound))]),
        };
        assert_eq!(a.clone().add(b.clone()), add_result);
        assert_eq!(a.sub(b), sub_result);
    }
}
