use super::LinearPolynomial;
use crate::domain::{
    bitvector::{RBound, concr::ConcreteBitvector},
    traits::forward::HwArith,
};

impl LinearPolynomial {
    pub fn arith_neg(mut self) -> LinearPolynomial {
        self.constant_term = self.constant_term.arith_neg();
        for monomial in self.linear_terms.iter_mut() {
            monomial.coefficient = monomial.coefficient.arith_neg();
        }

        self.into_normal_form()
    }

    pub fn add(self, rhs: LinearPolynomial) -> LinearPolynomial {
        //eprintln!("Adding {:?} and {:?}", self, rhs);
        // the main polynomial combination function

        // combine the constants
        let constant_term = self.constant_term.add(rhs.constant_term);

        // combine the polynomials in interleaved fashion based on slices

        let mut lhs_iter = self.linear_terms.into_iter().peekable();
        let mut rhs_iter = rhs.linear_terms.into_iter().peekable();

        let mut linear_terms = Vec::new();

        while let (Some(lhs_peek), Some(rhs_peek)) = (lhs_iter.peek(), rhs_iter.peek()) {
            // two competing monomials
            match lhs_peek.slice.cmp(&rhs_peek.slice) {
                std::cmp::Ordering::Less => {
                    // lhs slice is lesser, push it
                    linear_terms.push(lhs_iter.next().expect("Peeked monomial should be present"));
                }
                std::cmp::Ordering::Greater => {
                    // rhs slice is lesser, push it
                    linear_terms.push(rhs_iter.next().expect("Peeked monomial should be present"));
                }
                std::cmp::Ordering::Equal => {
                    // both slices are equal, add the coefficients of both (advanced)
                    let mut monomial = lhs_iter.next().expect("Peeked monomial should be present");
                    let rhs_monomial = rhs_iter.next().expect("Peeked monomial should be present");
                    monomial.coefficient = monomial.coefficient.add(rhs_monomial.coefficient);
                    linear_terms.push(monomial);
                }
            }
        }

        // extend the linear terms by the iterators to preserve the remainder
        linear_terms.extend(lhs_iter);
        linear_terms.extend(rhs_iter);

        // construct the polynomial and convert it to normal form

        let polynomial = LinearPolynomial {
            constant_term,
            linear_terms,
        };
        //eprintln!("Result polynomial: {:?}", polynomial);
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

        self.constant_term = self.constant_term.mul(scaler);

        for monomial in self.linear_terms.iter_mut() {
            monomial.coefficient = monomial.coefficient.mul(scaler);
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
