use std::path::PathBuf;

use crate::{
    args::SolverMode,
    domain::value::ThreeValued,
    problem::Problem,
    solver::internal::{InternalSolver, roole::RooleLearned},
};

mod internal;

mod preprocess;

#[derive(Debug)]
pub struct SolverSettings {
    /// Directory in which to place output artefacts.
    pub output_dir: Option<PathBuf>,
    /// File in which to write the proof.
    pub proof_output: Option<PathBuf>,
    /// Which solver mode to use.
    pub solver_mode: SolverMode,
    /// Whether preprocessing should be used.
    pub preprocess: bool,
    /// Whether to debug-print in hexadecimal mode.
    pub hexadecimal: bool,
}

pub fn solve(problem: &Problem, settings: &SolverSettings) -> ThreeValued {
    let preprocessed = if settings.preprocess {
        Some(preprocess::preprocess(problem, settings))
    } else {
        None
    };

    let problem = preprocessed.as_ref().unwrap_or(problem);

    // process
    let solution = match settings.solver_mode {
        SolverMode::Internal => {
            let solver: InternalSolver<'_, RooleLearned> = InternalSolver::new(
                problem,
                settings.output_dir.as_ref(),
                settings.proof_output.as_ref(),
            );
            solver.solve()
        }
        SolverMode::None => return problem.trivial_result(),
    };

    ThreeValued::from_bool(solution.result())
}
