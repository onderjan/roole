use std::path::PathBuf;

use crate::{
    problem::Problem,
    solver::internal::{InternalSolver, roole::RooleLearned},
};

mod internal;

pub fn solve(problem: &Problem, output_dir: Option<PathBuf>) {
    let solver: InternalSolver<'_, RooleLearned> = InternalSolver::new(problem, output_dir);
    solver.solve();
}
