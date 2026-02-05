mod monomial;
mod polynomial;
mod relation;
mod slice;
mod system;

use std::{collections::BTreeMap, fmt::Debug};

use crate::{
    domain::bitvector::{BitvectorBound, RBound, concr::ConcreteBitvector},
    problem::{eval::EvaluableDomain, formula::FormulaId},
};
use serde::{Deserialize, Serialize};

pub use {polynomial::LinearPolynomial, relation::LinearRelation, system::LinearSystem};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinearOperation(LinearOperationType);

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum LinearOperationType {
    Polynomial(LinearPolynomial),
    System(LinearSystem),
}

impl LinearOperation {
    pub fn evaluate<D: EvaluableDomain>(&self, fetch: impl Fn(FormulaId) -> D) -> D {
        match &self.0 {
            LinearOperationType::Polynomial(polynomial) => polynomial.evaluate(fetch),
            LinearOperationType::System(system) => system.evaluate(fetch),
        }
    }

    pub fn from_polynomial(polynomial: LinearPolynomial) -> Self {
        Self(LinearOperationType::Polynomial(polynomial))
    }

    pub fn from_system(system: LinearSystem) -> Self {
        // try to convert to polynomial
        match system.try_into_polynomial() {
            Ok(polynomial) => Self::from_polynomial(polynomial),
            Err(system) => Self(LinearOperationType::System(system)),
        }
    }

    pub fn try_into_polynomial(self) -> Result<LinearPolynomial, LinearOperation> {
        match self.0 {
            LinearOperationType::Polynomial(polynomial) => Ok(polynomial),
            ty => Err(Self(ty)),
        }
    }

    pub fn constant_value(&self) -> Option<ConcreteBitvector<RBound>> {
        match &self.0 {
            LinearOperationType::Polynomial(polynomial) => polynomial.constant_value(),
            _ => None,
        }
    }

    pub(crate) fn into_type(self) -> LinearOperationType {
        self.0
    }

    pub fn result_bound(&self) -> RBound {
        match &self.0 {
            LinearOperationType::Polynomial(polynomial) => polynomial.bound(),
            LinearOperationType::System(_) => RBound::single_bit_bound(),
        }
    }

    pub fn used_ids(&self) -> Vec<FormulaId> {
        match &self.0 {
            LinearOperationType::Polynomial(polynomial) => polynomial.used_ids(),
            LinearOperationType::System(system) => system.used_ids(),
        }
    }

    pub fn remap(&mut self, old_to_new: &BTreeMap<FormulaId, FormulaId>) {
        match &mut self.0 {
            LinearOperationType::Polynomial(polynomial) => polynomial.remap(old_to_new),
            LinearOperationType::System(system) => system.remap(old_to_new),
        }
    }

    pub fn bit_not(self) -> Self {
        match self.0 {
            LinearOperationType::Polynomial(polynomial) => {
                Self(LinearOperationType::Polynomial(polynomial.bit_not()))
            }
            LinearOperationType::System(system) => match system.bit_not() {
                Ok(system) => Self::from_system(system),
                Err(constant) => Self(LinearOperationType::Polynomial(
                    LinearPolynomial::single_bit(constant),
                )),
            },
        }
    }
}

impl Debug for LinearOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            LinearOperationType::Polynomial(polynomial) => Debug::fmt(polynomial, f),
            LinearOperationType::System(system) => Debug::fmt(system, f),
        }
    }
}
