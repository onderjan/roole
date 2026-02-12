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

#[derive(Debug)]
pub struct SolverSettings {
    /// Directory in which to place output artefacts.
    pub output_dir: Option<PathBuf>,
    /// Which solver mode to use.
    pub solver_mode: SolverMode,
    /// Whether preprocessing should be used.
    pub preprocess: bool,
    /// Whether to debug-print in hexadecimal mode.
    pub hexadecimal: bool,
}

pub fn solve(problem: &Problem, settings: &SolverSettings) {
    let preprocessed = if settings.preprocess {
        Some(preprocess::preprocess(problem, settings))
    } else {
        None
    };

    let problem = preprocessed.as_ref().unwrap_or(problem);

    // process
    match settings.solver_mode {
        SolverMode::Internal => {
            let solver: InternalSolver<'_, RooleLearned> =
                InternalSolver::new(problem, settings.output_dir.as_ref());
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
        SolverMode::None => {
            // do nothing
        }
    }
}
