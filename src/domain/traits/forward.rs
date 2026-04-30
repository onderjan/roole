use crate::domain::bitvector::BitvectorBound;

pub trait TypedEq {
    type Output;

    #[must_use]
    fn eq(self, rhs: Self) -> Self::Output;

    #[must_use]
    fn ne(self, rhs: Self) -> Self::Output;

    fn ite(condition: Self::Output, then_branch: Self, else_branch: Self) -> Self;
}

pub trait TypedCmp {
    type Output;

    #[must_use]
    fn ult(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn slt(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn ule(self, rhs: Self) -> Self::Output;
    #[must_use]
    fn sle(self, rhs: Self) -> Self::Output;
}

pub trait Bitwise
where
    Self: Sized,
{
    #[must_use]
    fn bit_not(self) -> Self;
    #[must_use]
    fn bit_and(self, rhs: Self) -> Self;
    #[must_use]
    fn bit_or(self, rhs: Self) -> Self;
    #[must_use]
    fn bit_xor(self, rhs: Self) -> Self;
}

pub trait HwArith
where
    Self: Sized,
{
    /// Arithmetic negation in two's complement.
    ///
    /// This is (2<sup>n</sup> - X) mod 2<sup>N</sup>,
    /// where the modulo only serves to return 2<sup>N</sup> to 0.
    ///
    /// The values 0 and 2<sup>n-1</sup> do not change.
    /// In the other values, the sign (most significant bit) is flipped.
    ///
    /// Notably, bitwise NOT is 2<sup>n</sup> - 1 - X.
    /// As such, we can convert `x.arith_neg().sub(1)` to `x.bit_not()`,
    /// and `x.bit_not().add(1)` to `x.arith_neg()`.
    #[must_use]
    fn arith_neg(self) -> Self;

    /// Wrapping addition of bitvectors.
    #[must_use]
    fn add(self, rhs: Self) -> Self;

    /// Wrapping subtraction in two's complement.
    #[must_use]
    fn sub(self, rhs: Self) -> Self;

    /// Wrapping multiplication.
    #[must_use]
    fn mul(self, rhs: Self) -> Self;

    /// Wrapping unsigned division that returns all ones on division by zero.
    ///
    /// The behaviour is as in SMT-LIB2 'bvudiv'.
    /// There are two special cases.
    /// Division by zero returns a bit-vector with all ones set (i.e. minus one).
    /// For overhalf 2<sup>n-1</sup>, division by (-1) wraps and returns the overhalf.
    #[must_use]
    fn udiv_wrapping_or_all_ones(self, rhs: Self) -> Self;

    /// Wrapping unsigned remainder that returns the dividend on division by zero.
    ///
    /// The behaviour is as in SMT-LIB2 'bvurem'.
    /// There are two special cases.
    /// Division by zero returns the dividend.
    /// For overhalf 2<sup>n-1</sup>, remainder by (-1) returns 0.
    #[must_use]
    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self;

    /// Wrapping signed division that uses unsigned division (wrapping or all-ones) in quadrants.
    ///
    /// The behaviour is as in SMT-LIB2 'bvsdiv'.
    /// This is specified by performing unsigned division (wrapping or all-ones)
    /// with arithmetic negations as follows (+/- signifies most significant bit being 0/1):
    /// (+) lhs, (+) rhs: lhs / rhs
    /// (-) lhs, (+) rhs: -((-lhs) / rhs)
    /// (+) lhs, (-) rhs: -(lhs / (-rhs))
    /// (-) lhs, (-) rhs: (-lhs) / (-rhs)
    ///
    /// The case of (2<sup>n-1</sup> sdiv -1) resolves to 2<sup>n-1</sup>,
    /// as expected of wrapping division.
    ///
    /// The results of division by zero are more complicated.
    /// For every X, unsigned division sets (X / 0) = -1.
    /// The case of (X sdiv 0) for non-negative X resolves to (X / 0) = -1.
    /// The case of (X sdiv 0) for negative X resolves to -((-X) / 0) = -(-1) = 1.
    #[must_use]
    fn sdiv_wrapping_by_quadrants(self, rhs: Self) -> Self;

    /// Wrapping signed remainder with sign following dividend
    /// that uses unsigned remainder (wrapping or all-ones) in quadrants.
    ///
    /// The behaviour is as in SMT-LIB2 'bvsrem'.
    /// This is specified by performing unsigned remainder (wrapping or all-ones)
    /// with arithmetic negations as follows (+/- signifies most significant bit being 0/1):
    /// (+) lhs, (+) rhs: lhs % rhs
    /// (-) lhs, (+) rhs: -((-lhs) % rhs)
    /// (+) lhs, (-) rhs: lhs % (-rhs)
    /// (-) lhs, (-) rhs: -((-lhs) % (-rhs))
    ///
    /// The case of (2<sup>n-1</sup> smod -1) resolves to 0, as expected of wrapping remainder.
    /// Remainder by zero returns the dividend.
    #[must_use]
    fn srem_wrapping_by_quadrants(self, rhs: Self) -> Self;

    /// Wrapping signed remainder with sign following divisor
    /// that uses unsigned remainder (wrapping or all-ones) in quadrants.
    ///
    /// The behaviour is as in SMT-LIB2 'bvsmod'.
    /// See the full definition at: https://smt-lib.org/logics-all.shtml#QF_BV
    #[must_use]
    fn smod_wrapping_by_quadrants(self, rhs: Self) -> Self;
}

pub trait HwShift {
    type Output;

    #[must_use]
    fn logic_shl(self, amount: Self) -> Self::Output;
    #[must_use]
    fn logic_shr(self, amount: Self) -> Self::Output;
    #[must_use]
    fn arith_shr(self, amount: Self) -> Self::Output;
}

pub trait Ext<const M: u32> {
    type Output;

    #[must_use]
    fn uext(self) -> Self::Output;
    #[must_use]
    fn sext(self) -> Self::Output;
}

pub trait BExt<X: BitvectorBound> {
    type Output;
    #[must_use]
    fn uext(self, new_bound: X) -> Self::Output;
    #[must_use]
    fn sext(self, new_bound: X) -> Self::Output;
}
