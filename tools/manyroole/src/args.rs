use std::path::PathBuf;

use clap::Parser;
use roole::args::{DEFAULT_SOLVER_MODE, SolverMode};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct ManyRooleArgs {
    /// Directory in which the outputs will be put.
    #[arg(long, default_value = "output/manyroole")]
    pub output_dir: PathBuf,

    /// Whether to retain the output directory or delete it beforehand.
    #[arg(long)]
    pub retain_output_dir: bool,

    /// The solver to use.
    #[arg(long, default_value_t = DEFAULT_SOLVER_MODE)]
    pub solver: SolverMode,

    /// Name of the instance for summary printing.
    #[arg(long)]
    pub instance_name: Option<String>,

    /// Input directory that will be taken as a root of the input paths for writing output files.
    #[arg(long)]
    pub input_root: Option<PathBuf>,

    /// SMT-LIB2 files or directories to process.
    #[structopt(required = true)]
    pub input_paths: Vec<PathBuf>,

    /// The Roole binary to use. If not set, `cargo run --package roole` will be used.
    #[arg(long)]
    pub roole_binary: Option<PathBuf>,

    #[arg(long)]
    pub silent: bool,

    /// Number of worker threads that will be used for parallel computation.
    #[arg(long)]
    pub num_workers: Option<u32>,
}
