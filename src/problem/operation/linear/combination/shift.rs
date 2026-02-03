use std::{collections::BTreeMap, num::NonZero};

use itertools::Itertools;

use crate::{
    domain::{bitvector::concr::ConcreteBitvector, traits::forward::HwShift},
    problem::operation::LinearCombination,
};

impl LinearCombination {
    pub fn logic_shl(self, _amount: Self) -> Result<Self, ()> {
        // TODO: logical shift left
        Err(())
    }

    pub fn logic_shr(mut self, amount: Self) -> Result<Self, ()> {
        let bound = self.bound();
        assert_eq!(self.bound(), amount.bound());

        let Some(amount) = amount.constant_value() else {
            return Err(());
        };

        if self.might_overflow() {
            return Err(());
        }

        // amount is constant and the linear combination cannot overflow
        if self.monomials.is_empty() {
            // we can simply shift the constant right by the amount
            self.constant = self.constant.logic_shr(amount);
            return Ok(self);
        }

        let Ok((mut slice, factor)) = self.monomials.into_iter().exactly_one() else {
            return Err(());
        };

        // TODO: handle other factors
        if !factor.is_one() {
            return Err(());
        }

        let Ok(amount) = u32::try_from(amount.to_u64()) else {
            // the shift amount is greater than maximum representable width
            // this will clearly make the combination empty
            return Ok(Self::empty(bound));
        };

        eprintln!("{:?} >> {:?}", slice, amount);

        // our combination only contains the slice

        if amount < slice.width.get() {
            // we will drop the lowest bits by increasing lsb
            // the width must decrease correspondingly
            slice.lsb += amount;
            slice.width = NonZero::new(slice.width.get() - amount)
                .expect("Slice width should be nonzero after logical shift right");

            eprintln!("New slice: {:?}", slice);

            Ok(Self::new(
                ConcreteBitvector::zero(bound),
                BTreeMap::from_iter([(slice, factor)]),
            ))
        } else {
            // all bits will be dropped
            Ok(Self::empty(bound))
        }
    }

    pub fn arith_shr(self, _amount: Self) -> Result<Self, ()> {
        // TODO: arithmetic shift right
        Err(())
    }
}
