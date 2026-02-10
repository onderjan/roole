use super::{
    super::{LinearPolynomial, LinearRelation},
    LinearExpression,
};
use crate::domain::{
    bitvector::concr::ConcreteBitvector,
    traits::forward::{Bitwise, HwArith},
};

impl LinearExpression {
    pub fn ult(self, rhs: Self) -> Result<Self, ()> {
        // resolve if either lhs or rhs is constant

        if let Some(lhs) = self.constant_value()
            && let LinearExpression::Polynomial(rhs) = rhs
        {
            // we have lhs < rhs where lhs is constant
            // we need to put the constant on the right side
            // flip comparison by turning each side to 2^N-side-1, i.e. !side
            // this gives us !rhs < !lhs
            // to turn this into less-or-equal, we need to test !lhs
            // if !lhs is zero, this is a contradiction
            // otherwise, we return !rhs <= !lhs - 1

            let not_lhs = lhs.bit_not();
            if not_lhs.is_zero() {
                // contradiction
                return Ok(Self::Polynomial(LinearPolynomial::from_bool(false)));
            }

            let polynomial = rhs.bit_not();
            let slack = not_lhs.sub(ConcreteBitvector::one(not_lhs.bound()));

            let relation = LinearRelation::new(polynomial, slack);

            return Ok(Self::Relation(relation).into_normal_form());
        }

        if let Some(rhs) = rhs.constant_value()
            && let LinearExpression::Polynomial(lhs) = self
        {
            // we have lhs < rhs where rhs is constant
            // we need to turn this into less-or-equal
            // if rhs is zero, this is a contradiction
            // otherwise, we return lhs <= rhs - 1

            if rhs.is_zero() {
                // contradiction
                return Ok(Self::Polynomial(LinearPolynomial::from_bool(false)));
            }

            let polynomial = lhs;
            let slack = rhs.sub(ConcreteBitvector::one(rhs.bound()));

            let relation = LinearRelation::new(polynomial, slack);

            return Ok(Self::Relation(relation).into_normal_form());
        }

        Err(())
    }

    pub fn ule(self, rhs: Self) -> Result<Self, ()> {
        // resolve if either lhs or rhs is constant

        if let Some(lhs) = self.constant_value()
            && let LinearExpression::Polynomial(rhs) = rhs
        {
            // we have lhs <= rhs where lhs is constant
            // we need to put the constant on the right side
            // flip comparison by turning each side to 2^N-side-1, i.e. !side
            // this gives us !rhs <= !lhs, which we want
            let polynomial = rhs.bit_not();
            let slack = lhs.bit_not();

            let relation = LinearRelation::new(polynomial, slack);

            return Ok(Self::Relation(relation).into_normal_form());
        }

        if let Some(rhs) = rhs.constant_value()
            && let LinearExpression::Polynomial(lhs) = self
        {
            // we have lhs <= rhs where rhs is constant
            // this is immediately expressible

            let polynomial = lhs;
            let slack = rhs;
            let relation = LinearRelation::new(polynomial, slack);

            return Ok(Self::Relation(relation).into_normal_form());
        }

        Err(())
    }
}
