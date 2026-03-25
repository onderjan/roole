use std::{fmt::Debug, path::Path};

use crate::{
    domain::bitvector::{
        abstr::{BitvectorDomain, RBitvector},
        concr::ConcreteBitvector,
    },
    problem::{
        Evaluator,
        assignment::Assignment,
        evaluator::EvaluableDomain,
        solution::proof::{Proof, UnsatValidator},
    },
};

pub mod proof;

use super::Problem;

/// A solution of the satisfiability problem.
///
/// It either says the problem is satisfiable, producing
/// an assignment that satisfies the problem (a model),
/// or says that the problem is unsatisfiable, producing
/// an unsatisfiability proof.
pub enum Solution<D: EvaluableDomain> {
    Satisfiable(Assignment<D>),
    Unsatisfiable(Proof),
}

impl Solution<RBitvector> {
    /// Validates (proof-checks) that the solution to a problem is correct.
    pub fn validate(&self, problem: &Problem) {
        match self {
            Solution::Satisfiable(claimed_sat_assignment) => {
                // it is claimed the assignment satisfies the problem formula
                // just evaluate the assignment and validate it returns single-bit bitvector 1
                let eval_result = Evaluator::new(problem).evaluate(claimed_sat_assignment);
                assert_eq!(
                    eval_result.concrete_value(),
                    Some(ConcreteBitvector::from_bool(true))
                );
            }
            Solution::Unsatisfiable(unsat_proof) => {
                // validate the proof
                UnsatValidator::new(problem, unsat_proof).validate();
            }
        }
    }

    pub fn result(&self) -> bool {
        match self {
            Solution::Satisfiable(_) => true,
            Solution::Unsatisfiable(_) => false,
        }
    }

    pub fn write_smt_proof(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(path)?;
        let proof = match self {
            Solution::Satisfiable(assignment) => &Proof::from_satisfying(assignment),
            Solution::Unsatisfiable(proof) => proof,
        };
        proof.write_smt(&mut file, self.result())?;
        Ok(())
    }
}

impl<D: EvaluableDomain> Debug for Solution<D>
where
    Assignment<D>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Satisfiable(arg0) => f.debug_tuple("Satisfiable").field(arg0).finish(),
            Self::Unsatisfiable(arg0) => f.debug_tuple("Unsatisfiable").field(arg0).finish(),
        }
    }
}
