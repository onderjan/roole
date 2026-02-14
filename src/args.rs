use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Directory in which to place output artefacts.
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,

    pub input_file: PathBuf,

    #[arg(short, long, default_value_t = SolverMode::Internal)]
    pub solver: SolverMode,

    #[arg(short, long)]
    pub preprocess: bool,

    #[arg(short = 'H', long)]
    pub hexadecimal: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum SolverMode {
    /// Use the internal solver
    Internal,
    /// Use the CaDiCaL solver
    Cadical,
    /// Do not use any solver
    None,
}

impl Display for SolverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SolverMode::Internal => "internal",
                SolverMode::Cadical => "cadical",
                SolverMode::None => "none",
            }
        )
    }
}
