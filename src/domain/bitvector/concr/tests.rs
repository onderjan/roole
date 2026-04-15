use crate::domain::{
    bitvector::{CBound, concr::ConcreteBitvector},
    traits::forward::{Bitwise, Ext, HwArith, HwShift, TypedCmp, TypedEq},
};

type CConcreteBitvector<const W: u32> = ConcreteBitvector<CBound<W>>;

#[test]
fn support() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);

    let zero = CConcreteBitvector::<16>::new(0, CBound);
    let full = CConcreteBitvector::<16>::new(0xFFFF, CBound);
    let min = CConcreteBitvector::<16>::new(0x8000, CBound);

    assert_eq!(a.try_to_u64().unwrap(), 0xCAFE);
    assert_eq!(b.try_to_u64().unwrap(), 0x1337);
    assert_eq!(zero.try_to_u64().unwrap(), 0);
    assert_eq!(full.try_to_u64().unwrap(), 0xFFFF);
    assert_eq!(min.try_to_u64().unwrap(), 0x8000);

    assert_eq!(a.try_to_i64().unwrap(), -0x3502);
    assert_eq!(b.try_to_i64().unwrap(), 0x1337);
    assert_eq!(zero.try_to_i64().unwrap(), 0);
    assert_eq!(full.try_to_i64().unwrap(), -1);
    assert_eq!(min.try_to_i64().unwrap(), -0x8000);

    assert!(a.is_nonzero());
    assert!(b.is_nonzero());
    assert!(!zero.is_nonzero());
    assert!(full.is_nonzero());
    assert!(min.is_nonzero());

    assert!(!a.is_zero());
    assert!(!b.is_zero());
    assert!(zero.is_zero());
    assert!(!full.is_zero());
    assert!(!min.is_zero());

    assert!(a.is_sign_bit_set());
    assert!(!b.is_sign_bit_set());
    assert!(!zero.is_sign_bit_set());
    assert!(full.is_sign_bit_set());
    assert!(min.is_sign_bit_set());

    assert_eq!(
        CConcreteBitvector::<8>::all_with_bound_iter(CBound).count(),
        2usize.pow(8)
    );
}

#[test]
fn eq() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);

    assert!(a.clone().eq(a.clone()).into_bool());
    assert!(b.clone().eq(b.clone()).into_bool());
    assert!(!a.clone().eq(b.clone()).into_bool());
    assert!(!b.clone().eq(a.clone()).into_bool());
}

#[test]
fn cmp() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);

    // identity
    assert!(!a.clone().ult(a.clone()).into_bool());
    assert!(a.clone().ule(a.clone()).into_bool());
    assert!(!a.clone().slt(a.clone()).into_bool());
    assert!(a.clone().sle(a.clone()).into_bool());

    // comparison
    assert!(!a.clone().ult(b.clone()).into_bool());
    assert!(!a.clone().ule(b.clone()).into_bool());
    assert!(a.clone().slt(b.clone()).into_bool());
    assert!(a.clone().sle(b.clone()).into_bool());

    // try flipped the other way, they are not equal
    assert!(b.clone().ult(a.clone()).into_bool());
    assert!(b.clone().ule(a.clone()).into_bool());
    assert!(!b.clone().slt(a.clone()).into_bool());
    assert!(!b.clone().sle(a.clone()).into_bool());
}

#[test]
fn bitwise() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);

    // compare results to calculated values
    assert_eq!(a.clone().bit_not().try_to_u64().unwrap(), 0x3501);
    assert_eq!(b.clone().bit_not().try_to_u64().unwrap(), 0xECC8);

    assert_eq!(a.clone().bit_and(b.clone()).try_to_u64().unwrap(), 0x0236);
    assert_eq!(a.clone().bit_or(b.clone()).try_to_u64().unwrap(), 0xDBFF);
    assert_eq!(a.clone().bit_xor(b.clone()).try_to_u64().unwrap(), 0xD9C9);
}

#[test]
fn ext() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);

    // longer uext will preserve unsigned value
    assert_eq!(Ext::<32>::uext(a.clone()).try_to_u64().unwrap(), 0xCAFE);
    assert_eq!(Ext::<32>::uext(a.clone()).try_to_i64().unwrap(), 0xCAFE);
    assert_eq!(Ext::<32>::uext(b.clone()).try_to_u64().unwrap(), 0x1337);
    assert_eq!(Ext::<32>::uext(b.clone()).try_to_i64().unwrap(), 0x1337);

    // longer sext will preserve signed value
    assert_eq!(Ext::<32>::sext(a.clone()).try_to_u64().unwrap(), 0xFFFFCAFE);
    assert_eq!(Ext::<32>::sext(a.clone()).try_to_i64().unwrap(), -0x3502);
    assert_eq!(Ext::<32>::sext(b.clone()).try_to_u64().unwrap(), 0x1337);
    assert_eq!(Ext::<32>::sext(b.clone()).try_to_i64().unwrap(), 0x1337);

    // shorter ext will always just cut
    assert_eq!(Ext::<4>::uext(a.clone()).try_to_u64().unwrap(), 0xE);
    assert_eq!(Ext::<4>::uext(a.clone()).try_to_i64().unwrap(), -0x2);
    assert_eq!(Ext::<4>::sext(a.clone()).try_to_u64().unwrap(), 0xE);
    assert_eq!(Ext::<4>::sext(a.clone()).try_to_i64().unwrap(), -0x2);

    // same ext will preserve value
    assert_eq!(a, Ext::<16>::uext(a.clone()));
    assert_eq!(b, Ext::<16>::uext(b.clone()));
    assert_eq!(a, Ext::<16>::sext(a.clone()));
    assert_eq!(b, Ext::<16>::sext(b.clone()));
}

#[test]
fn shift() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);
    let four = CConcreteBitvector::<16>::new(0x4, CBound);
    let sixteen = CConcreteBitvector::<16>::new(0x16, CBound);
    let too_much = CConcreteBitvector::<16>::new(0x42, CBound);

    // shift by a reasonable value
    assert_eq!(
        a.clone().logic_shl(four.clone()).try_to_u64().unwrap(),
        0xAFE0
    );
    assert_eq!(
        b.clone().logic_shl(four.clone()).try_to_u64().unwrap(),
        0x3370
    );
    assert_eq!(
        a.clone().logic_shr(four.clone()).try_to_u64().unwrap(),
        0x0CAF
    );
    assert_eq!(
        b.clone().logic_shr(four.clone()).try_to_u64().unwrap(),
        0x0133
    );
    assert_eq!(
        a.clone().arith_shr(four.clone()).try_to_u64().unwrap(),
        0xFCAF
    );
    assert_eq!(
        b.clone().arith_shr(four.clone()).try_to_u64().unwrap(),
        0x0133
    );

    // shift by exactly all bits
    // should be zero except for arith shift right of negative
    assert_eq!(
        a.clone().logic_shl(sixteen.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        b.clone().logic_shl(sixteen.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        a.clone().logic_shr(sixteen.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        b.clone().logic_shr(sixteen.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        a.clone().arith_shr(sixteen.clone()).try_to_u64().unwrap(),
        0xFFFF
    );
    assert_eq!(
        b.clone().arith_shr(sixteen.clone()).try_to_u64().unwrap(),
        0x0000
    );

    // shift by an unreasonable value
    // should give the same results as by all bits
    assert_eq!(
        a.clone().logic_shl(too_much.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        b.clone().logic_shl(too_much.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        a.clone().logic_shr(too_much.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        b.clone().logic_shr(too_much.clone()).try_to_u64().unwrap(),
        0x0000
    );
    assert_eq!(
        a.clone().arith_shr(too_much.clone()).try_to_u64().unwrap(),
        0xFFFF
    );
    assert_eq!(
        b.clone().arith_shr(too_much.clone()).try_to_u64().unwrap(),
        0x0000
    );
}

#[test]
fn arith() {
    let a = CConcreteBitvector::<16>::new(0xCAFE, CBound);
    let b = CConcreteBitvector::<16>::new(0x1337, CBound);
    let c = CConcreteBitvector::<16>::new(0x4000, CBound);
    let d = CConcreteBitvector::<16>::new(0xBADA, CBound);

    let zero = CConcreteBitvector::<16>::new(0, CBound);

    // add, sub, mul do not have corner cases except wrapping
    // compare results to calculated values
    assert_eq!(a.clone().add(b.clone()).try_to_u64().unwrap(), 0xDE35);
    assert_eq!(a.clone().add(c.clone()).try_to_u64().unwrap(), 0x0AFE);
    assert_eq!(b.clone().add(c.clone()).try_to_u64().unwrap(), 0x5337);
    assert_eq!(a.clone().sub(b.clone()).try_to_u64().unwrap(), 0xB7C7);
    assert_eq!(a.clone().sub(c.clone()).try_to_u64().unwrap(), 0x8AFE);
    assert_eq!(b.clone().sub(c.clone()).try_to_u64().unwrap(), 0xD337);
    assert_eq!(a.clone().mul(b.clone()).try_to_u64().unwrap(), 0x7692);
    assert_eq!(a.clone().mul(c.clone()).try_to_u64().unwrap(), 0x8000);
    assert_eq!(b.clone().mul(c.clone()).try_to_u64().unwrap(), 0xC000);

    // unsigned division and remainder have division by zero
    // try out normal first
    assert_eq!(
        a.clone()
            .udiv_wrapping_or_all_ones(b.clone())
            .try_to_u64()
            .unwrap(),
        0x000A
    );
    assert_eq!(
        a.clone()
            .urem_wrapping_or_dividend(b.clone())
            .try_to_u64()
            .unwrap(),
        0x0AD8
    );
    assert_eq!(
        a.clone()
            .udiv_wrapping_or_all_ones(c.clone())
            .try_to_u64()
            .unwrap(),
        0x0003
    );
    assert_eq!(
        a.clone()
            .urem_wrapping_or_dividend(c.clone())
            .try_to_u64()
            .unwrap(),
        0x0AFE
    );
    assert_eq!(
        b.clone()
            .udiv_wrapping_or_all_ones(c.clone())
            .try_to_u64()
            .unwrap(),
        0x0000
    );
    assert_eq!(
        b.clone()
            .urem_wrapping_or_dividend(c.clone())
            .try_to_u64()
            .unwrap(),
        0x1337
    );

    // in case of unsigned division-by-zero
    // division result is all-ones and remainder is the dividend
    assert_eq!(
        a.clone()
            .udiv_wrapping_or_all_ones(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFF
    );
    assert_eq!(
        a.clone()
            .urem_wrapping_or_dividend(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xCAFE
    );
    assert_eq!(
        b.clone()
            .udiv_wrapping_or_all_ones(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFF
    );
    assert_eq!(
        b.clone()
            .urem_wrapping_or_dividend(zero.clone())
            .try_to_u64()
            .unwrap(),
        0x1337
    );
    assert_eq!(
        c.clone()
            .udiv_wrapping_or_all_ones(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFF
    );
    assert_eq!(
        c.clone()
            .urem_wrapping_or_dividend(zero.clone())
            .try_to_u64()
            .unwrap(),
        0x4000
    );

    // signed division and remainder have four-quadrant behaviour,
    // division by zero and overflow
    assert_eq!(
        c.clone()
            .sdiv_wrapping_by_quadrants(b.clone())
            .try_to_u64()
            .unwrap(),
        0x0003
    ); // positive / positive
    assert_eq!(
        c.clone()
            .srem_wrapping_by_quadrants(b.clone())
            .try_to_u64()
            .unwrap(),
        0x065B
    );
    assert_eq!(
        a.clone()
            .sdiv_wrapping_by_quadrants(b.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFE
    ); // negative / positive
    assert_eq!(
        a.clone()
            .srem_wrapping_by_quadrants(b.clone())
            .try_to_u64()
            .unwrap(),
        0xF16C
    );
    assert_eq!(
        c.clone()
            .sdiv_wrapping_by_quadrants(a.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFF
    ); // positive / negative
    assert_eq!(
        c.clone()
            .srem_wrapping_by_quadrants(a.clone())
            .try_to_u64()
            .unwrap(),
        0x0AFE
    );
    assert_eq!(
        d.clone()
            .sdiv_wrapping_by_quadrants(a.clone())
            .try_to_u64()
            .unwrap(),
        0x0001
    ); // negative / negative
    assert_eq!(
        d.clone()
            .srem_wrapping_by_quadrants(a.clone())
            .try_to_u64()
            .unwrap(),
        0xEFDC
    );

    // in case of signed division-by-zero
    // division result is all-ones for non-negative dividend
    // and one for negative dividend
    // remainder is the dividend
    assert_eq!(
        a.clone()
            .sdiv_wrapping_by_quadrants(zero.clone())
            .try_to_u64()
            .unwrap(),
        0x0001
    );
    assert_eq!(
        a.clone()
            .srem_wrapping_by_quadrants(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xCAFE
    );
    assert_eq!(
        b.clone()
            .sdiv_wrapping_by_quadrants(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFF
    );
    assert_eq!(
        b.clone()
            .srem_wrapping_by_quadrants(zero.clone())
            .try_to_u64()
            .unwrap(),
        0x1337
    );
    assert_eq!(
        c.clone()
            .sdiv_wrapping_by_quadrants(zero.clone())
            .try_to_u64()
            .unwrap(),
        0xFFFF
    );
    assert_eq!(
        c.clone()
            .srem_wrapping_by_quadrants(zero.clone())
            .try_to_u64()
            .unwrap(),
        0x4000
    );

    // overflow only happens if the minimum value is divided by minus one
    // because the minimum value is not representable in positive
    // in that case, we expect the minimum value remain in divisor
    // and no remainder
    let min = CConcreteBitvector::<16>::new(0x8000, CBound);
    let minus_one = CConcreteBitvector::<16>::new(0xFFFF, CBound);
    assert_eq!(
        min.clone().sdiv_wrapping_by_quadrants(minus_one.clone()),
        min
    );
    assert_eq!(
        min.clone().srem_wrapping_by_quadrants(minus_one.clone()),
        zero
    );
}
