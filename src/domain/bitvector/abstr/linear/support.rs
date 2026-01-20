use std::fmt::Debug;

use crate::domain::{
    bitvector::{
        BitvectorBound,
        abstr::linear::{LinearBitvector, LinearCombination},
    },
    traits::Join,
};

impl<B: BitvectorBound> Join for LinearBitvector<B> {
    fn join(self, other: &Self) -> Self {
        todo!()
    }

    fn apply_join(&mut self, other: &Self) {
        todo!()
    }

    fn contains(&self, contained: &Self) -> bool {
        todo!()
    }
}

impl<B: BitvectorBound> Debug for LinearBitvector<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(combination) = &self.combination {
            Debug::fmt(combination, f)
        } else {
            write!(f, "⊤")
        }
    }
}

impl Debug for LinearCombination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;

        // write the linear combinations of formulas with coefficients
        for (formula_id, coefficient) in &self.variables {
            if is_first {
                is_first = false;
            } else {
                write!(f, " + ")?;
            }
            write!(f, "({}*{:?})", coefficient, formula_id)?;
        }

        if is_first {
            write!(f, "({})", self.constant)?;
        } else if self.constant != 0 {
            write!(f, " + ({})", self.constant)?;
        }
        Ok(())
    }
}
