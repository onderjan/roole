use std::path::PathBuf;

use crate::{
    SolverMode,
    domain::bitvector::{RBound, abstr::linear::LinearBitvector},
    problem::{Evaluator, Problem},
    solver::internal::{InternalSolver, roole::RooleLearned},
};

#[cfg(feature = "cadical")]
mod cadical;
mod internal;

pub fn solve(
    problem: &Problem,
    output_dir: Option<PathBuf>,
    solver_mode: SolverMode,
    preprocess: bool,
) {
    if preprocess {
        // preprocess
        let mut preprocessor = Evaluator::<LinearBitvector<RBound>>::new(problem);

        let assignment = problem.linear_assignment();
        preprocessor.evaluate(&assignment);
        eprintln!("Preprocessor: {:#?}", preprocessor);
    }

    // process
    match solver_mode {
        SolverMode::Internal => {
            let solver: InternalSolver<'_, RooleLearned> = InternalSolver::new(problem, output_dir);
            solver.solve();
        }
        SolverMode::Cadical => {
            #[cfg(feature = "cadical")]
            {
                cadical::CadicalSolver::new(problem, output_dir).solve();
            };

            #[cfg(not(feature = "cadical"))]
            {
                panic!("CaDiCaL feature not enabled");
            };
        }
    }
}
