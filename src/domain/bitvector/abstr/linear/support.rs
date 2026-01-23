use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::linear::{
                LinearBitvector, LinearCombination, LinearRelation, LinearSystem, LinearType,
            },
            concr::ConcreteBitvector,
        },
        traits::Join,
    },
    problem::formula::FormulaId,
};

impl LinearBitvector {
    pub fn for_formula_id(formula_id: FormulaId, bound: RBound) -> Self {
        let constant = ConcreteBitvector::zero(bound);
        let mut coefficients = BTreeMap::new();
        coefficients.insert(formula_id, ConcreteBitvector::one(bound));

        LinearBitvector {
            bound,
            ty: LinearType::Combination(LinearCombination {
                constant,
                coefficients,
            }),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self.ty {
            LinearType::Top => vec![],
            LinearType::Combination(combination) => {
                combination.coefficients.keys().copied().collect()
            }
            LinearType::System(system) => system
                .equations
                .iter()
                .map(|equation| &equation.left)
                .flat_map(|combination| combination.coefficients.keys().copied())
                .collect(),
        }
    }
}

impl LinearCombination {
    pub(super) fn normalize(&mut self) {
        // eliminate zero coefficients
        self.coefficients.retain(|_, coeff| !coeff.is_zero());
    }
}

impl Join for LinearBitvector {
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

impl Debug for LinearBitvector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.ty {
            LinearType::Top => write!(f, "⊤"),
            LinearType::Combination(combination) => Debug::fmt(combination, f),
            LinearType::System(system) => Debug::fmt(system, f),
        }
    }
}

impl Debug for LinearCombination {
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

            let one = ConcreteBitvector::<RBound>::one(coefficient.bound());

            if coefficient != &one {
                write!(f, "{}*", coefficient)?;
            }

            write!(f, "{:?}", formula_id)?;
        }

        if is_first {
            write!(f, "{}", self.constant)?;
        } else if self.constant.is_nonzero() {
            write!(f, " + {}", self.constant)?;
        }
        write!(f, ") mod {}", 1u64 << self.constant.bound().width())
    }
}

impl Debug for LinearSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for equation in &self.equations {
            if is_first {
                is_first = false;
            } else {
                write!(f, " ∧ ")?;
            }
            Debug::fmt(&equation.left, f)?;
            let op = match equation.op {
                LinearRelation::Eq => "==",
                LinearRelation::Ne => "!=",
                LinearRelation::Lt => "<",
            };
            write!(f, " {} ", op)?;
            Debug::fmt(&equation.right, f)?;
        }
        Ok(())
    }
}
