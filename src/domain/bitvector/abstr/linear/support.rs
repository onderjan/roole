use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::{
        bitvector::{
            BitvectorBound,
            abstr::linear::{LinearBitvector, LinearCombination},
            concr::ConcreteBitvector,
        },
        traits::Join,
    },
    problem::formula::FormulaId,
};

impl<B: BitvectorBound> LinearBitvector<B> {
    pub fn for_formula_id(formula_id: FormulaId, bound: B) -> Self {
        let constant = ConcreteBitvector::zero(bound);
        let mut coefficients = BTreeMap::new();
        coefficients.insert(formula_id, ConcreteBitvector::one(bound));

        LinearBitvector {
            bound,
            combination: Some(LinearCombination {
                constant,
                coefficients,
            }),
        }
    }
}

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

impl<B: BitvectorBound> Debug for LinearCombination<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;

        write!(f, "(")?;

        // write the linear combinations of formulas with coefficients
        for (formula_id, coefficient) in &self.coefficients {
            if is_first {
                is_first = false;
            } else {
                write!(f, " + ")?;
            }
            write!(f, "{}*{:?}", coefficient, formula_id)?;
        }

        if is_first {
            write!(f, "{}", self.constant)?;
        } else if self.constant.is_nonzero() {
            write!(f, " + {}", self.constant)?;
        }
        write!(f, ") mod {}", 1u64 << self.constant.bound().width())
    }
}
