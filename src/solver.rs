use std::path::PathBuf;

use crate::{
    SolverMode,
    problem::Problem,
    solver::internal::{InternalSolver, roole::RooleLearned},
};

#[cfg(feature = "cadical")]
mod cadical;
mod internal;

mod preprocess;

pub fn solve(
    problem: &Problem,
    output_dir: Option<PathBuf>,
    solver_mode: SolverMode,
    preprocess: bool,
) {
    let preprocessed = if preprocess {
        Some(preprocess::preprocess(problem))
    } else {
        None
    };

    let problem = preprocessed.as_ref().unwrap_or(problem);

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
