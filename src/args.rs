use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Directory in which to place debug artefacts.
    #[arg(short, long)]
    pub debug_dir: Option<PathBuf>,

    /// Input SMT-LIB2 file.
    pub input_file: PathBuf,

    /// Path to which the proof certificate will be written.
    #[arg(short = 'P', long)]
    pub proof_output: Option<PathBuf>,

    /// Solver to use.
    #[arg(short, long, default_value_t = DEFAULT_SOLVER_MODE)]
    pub solver: SolverMode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SolverMode {
    /// Use the internal solver
    Internal,
    /// Do not use any solver
    None,
}

pub const DEFAULT_SOLVER_MODE: SolverMode = SolverMode::Internal;

impl Display for SolverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SolverMode::Internal => "internal",
                SolverMode::None => "none",
            }
        )
    }
}
