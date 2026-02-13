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
    #[must_use]
    fn arith_neg(self) -> Self;

    #[must_use]
    fn add(self, rhs: Self) -> Self;
    #[must_use]
    fn sub(self, rhs: Self) -> Self;
    #[must_use]
    fn mul(self, rhs: Self) -> Self;

    #[must_use]
    fn udiv_wrapping_or_full(self, rhs: Self) -> Self;
    #[must_use]
    fn sdiv_wrapping_or_full(self, rhs: Self) -> Self;

    #[must_use]
    fn urem_wrapping_or_dividend(self, rhs: Self) -> Self;
    #[must_use]
    fn srem_wrapping_or_dividend(self, rhs: Self) -> Self;
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
