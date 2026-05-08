use std::path::PathBuf;

use crate::{
    args::SolverMode,
    domain::value::ThreeValued,
    problem::Problem,
    solver::internal::{InternalSolver, roole::RooleLearned},
};

mod internal;

#[derive(Debug)]
pub struct SolverSettings {
    /// Directory in which to place debugging artefacts.
    pub debug_dir: Option<PathBuf>,
    /// File in which to write the proof.
    pub proof_output: Option<PathBuf>,
    /// Which solver mode to use.
    pub solver_mode: SolverMode,
}

pub fn solve(problem: &Problem, settings: &SolverSettings) -> ThreeValued {
    let solution = match settings.solver_mode {
        SolverMode::Internal => {
            let solver: InternalSolver<'_, RooleLearned> = InternalSolver::new(
                problem,
                settings.debug_dir.as_ref(),
                settings.proof_output.as_ref(),
            );
            solver.solve()
        }
        SolverMode::None => return problem.trivial_result(),
    };

    ThreeValued::from_bool(solution.result())
}
