use super::LinearPolynomial;
use crate::{
    domain::{
        bitvector::{RBound, concr::ConcreteBitvector},
        traits::forward::{Bitwise, HwArith, HwShift},
    },
    problem::symbolic::linear::{monomial::LinearMonomial, slice::LinearSlice},
};

impl LinearPolynomial {
    pub fn arith_neg(mut self) -> LinearPolynomial {
        self.constant_term = self.constant_term.arith_neg();
        for monomial in self.linear_terms.iter_mut() {
            monomial.coefficient = monomial.coefficient.clone().arith_neg();
        }

        self.into_normal_form()
    }

    pub fn add(self, rhs: LinearPolynomial) -> LinearPolynomial {
        // the main polynomial combination function

        let bound = self.bound();
        assert_eq!(bound, rhs.bound());

        // combine the constants
        let constant_term = self.constant_term.add(rhs.constant_term);

        // combine the polynomials in interleaved fashion based on slices

        let mut lhs_iter = self.linear_terms.into_iter();
        let mut rhs_iter = rhs.linear_terms.into_iter();

        let mut current_lhs = None;
        let mut current_rhs = None;

        let mut linear_terms = Vec::new();

        loop {
            if current_lhs.is_none() {
                current_lhs = lhs_iter.next();
            }

            if current_rhs.is_none() {
                current_rhs = rhs_iter.next();
            }
            // ensure we have both lhs and rhs to combine them

            let Some(lhs_monomial) = current_lhs.take() else {
                // no lhs monomial, push the current rhs if exists and break loop
                if let Some(rhs_monomial) = current_rhs.take() {
                    linear_terms.push(rhs_monomial);
                }
                break;
            };
            let Some(rhs_monomial) = current_rhs.take() else {
                // no rhs monomial, push the current lhs and break loop
                linear_terms.push(lhs_monomial);
                break;
            };

            // we now have both lhs and rhs and need to properly handle them

            if lhs_monomial.slice.formula_id != rhs_monomial.slice.formula_id {
                // slices have nothing in common
                // push lesser one, they are clearly unequal
                if lhs_monomial.slice <= rhs_monomial.slice {
                    // lhs slice is lesser, push lhs
                    linear_terms.push(lhs_monomial);
                    // put rhs back to current rhs and get next lhs
                    (current_lhs, current_rhs) = (lhs_iter.next(), Some(rhs_monomial));
                } else {
                    // rhs slice is lesser, push rhs
                    linear_terms.push(rhs_monomial);
                    // put lhs back to current lhs and get next rhs
                    (current_lhs, current_rhs) = (Some(lhs_monomial), rhs_iter.next());
                }
                continue;
            }

            let formula_id = lhs_monomial.slice.formula_id;

            // a.lsb <= b.lsb
            let (a, b, a_is_lhs) = if lhs_monomial.slice.lsb <= rhs_monomial.slice.lsb {
                (lhs_monomial, rhs_monomial, true)
            } else {
                (rhs_monomial, lhs_monomial, false)
            };

            let a_mask = a.slice.formula_mask(bound);
            let b_mask = b.slice.formula_mask(bound);

            let only_a = a_mask.clone().bit_and(b_mask.clone().bit_not());
            let a_and_b = a_mask.clone().bit_and(b_mask.clone());
            let only_b = b_mask.bit_and(a_mask.bit_not());

            // as they overlap, a_and_b is always nonzero
            // if nonzero: only_a < a_and_b, a_and_b < only_b, and only_a < only_b

            if only_a.is_nonzero() {
                // consume only_a
                // this must have the same coefficient as a
                let only_a = LinearMonomial::new(
                    a.coefficient.clone(),
                    LinearSlice::from_mask(formula_id, only_a),
                );

                linear_terms.push(only_a);
            }

            // consume a_and_b
            if a_and_b.is_nonzero() {
                // we need to scale the coefficients
                let a_and_b = LinearSlice::from_mask(formula_id, a_and_b);

                let scaling_a = a_and_b.lsb - a.slice.lsb;
                let scaling_b = a_and_b.lsb - b.slice.lsb;

                let a_bound = a.coefficient.bound();

                let scaled_coeff_a = a
                    .coefficient
                    .logic_shl(ConcreteBitvector::new(scaling_a.into(), a_bound));

                let b_bound = b.coefficient.bound();

                let scaled_coeff_b = b
                    .coefficient
                    .clone()
                    .logic_shl(ConcreteBitvector::new(scaling_b.into(), b_bound));

                let a_and_b_coeff = scaled_coeff_a.add(scaled_coeff_b);

                let a_and_b = LinearMonomial::new(a_and_b_coeff, a_and_b);
                linear_terms.push(a_and_b);
            }

            if only_b.is_nonzero() {
                // retain only_b
                let only_b = LinearSlice::from_mask(formula_id, only_b);
                let scaled_coeff_b = b.coefficient.clone().logic_shl(ConcreteBitvector::new(
                    (only_b.lsb - b.slice.lsb).into(),
                    b.coefficient.bound(),
                ));
                let only_b = LinearMonomial::new(scaled_coeff_b, only_b);

                if a_is_lhs {
                    current_rhs = Some(only_b);
                } else {
                    current_lhs = Some(only_b);
                }
            }
        }

        // extend the linear terms by the iterators to preserve the remainder
        linear_terms.extend(lhs_iter);
        linear_terms.extend(rhs_iter);

        /*if !linear_terms.is_sorted_by(|a, b| a.slice < b.slice) {
            panic!("Not sorted linear terms: {:?}", linear_terms);
        }*/

        // construct the polynomial and convert it to normal form

        let polynomial = LinearPolynomial {
            constant_term,
            linear_terms,
        };
        polynomial.into_normal_form()
    }

    pub fn sub(self, rhs: LinearPolynomial) -> LinearPolynomial {
        // subtract by adding negated rhs
        let rhs = rhs.arith_neg();
        self.add(rhs)
    }

    pub fn mul(self, rhs: LinearPolynomial) -> Result<LinearPolynomial, ()> {
        // we can only multiply if at least one of the polynomials is constant
        let (constant, mut polynomial) = if let Some(constant) = self.constant_value() {
            (constant, rhs)
        } else if let Some(constant) = rhs.constant_value() {
            (constant, self)
        } else {
            // neither is a constant
            return Err(());
        };

        // multiply polynomial by constant
        polynomial.scale(constant);
        Ok(polynomial)
    }

    pub fn scale(&mut self, scaler: ConcreteBitvector<RBound>) {
        let bound = self.bound();
        assert_eq!(bound, scaler.bound());

        self.constant_term = self.constant_term.clone().mul(scaler.clone());

        for monomial in self.linear_terms.iter_mut() {
            monomial.coefficient = monomial.coefficient.clone().mul(scaler.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use crate::{
        domain::bitvector::{RBound, concr::ConcreteBitvector},
        problem::{
            formula::{FormulaId, VariableId},
            symbolic::linear::monomial::LinearMonomial,
        },
    };

    use super::*;

    use super::super::LinearSlice;

    #[test]
    fn test_addsub() {
        let bound = RBound::new(32);
        let slice = LinearSlice {
            formula_id: FormulaId::Variable(VariableId(0)),
            lsb: 0,
            width: NonZero::new(32).unwrap(),
        };
        let a = LinearPolynomial::from_monomial_and_constant(
            LinearMonomial::new(ConcreteBitvector::new(12, bound), slice),
            ConcreteBitvector::new(38, bound),
        );
        let b = LinearPolynomial::from_monomial_and_constant(
            LinearMonomial::new(ConcreteBitvector::new(7, bound), slice),
            ConcreteBitvector::new(17, bound),
        );
        let add_result = LinearPolynomial::from_monomial_and_constant(
            LinearMonomial::new(ConcreteBitvector::new(19, bound), slice),
            ConcreteBitvector::new(55, bound),
        );
        let sub_result = LinearPolynomial::from_monomial_and_constant(
            LinearMonomial::new(ConcreteBitvector::new(5, bound), slice),
            ConcreteBitvector::new(21, bound),
        );
        assert_eq!(a.clone().add(b.clone()), add_result);
        assert_eq!(a.sub(b), sub_result);
    }
}
