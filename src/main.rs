use std::{fmt::Display, io::BufReader, path::PathBuf};

use clap::{Parser, ValueEnum};

mod domain;
mod parser;
mod problem;
mod solver;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory in which to place output artefacts.
    #[arg(short, long)]
    output_dir: Option<PathBuf>,

    input_file: PathBuf,

    #[arg(short, long, default_value_t = SolverMode::Internal)]
    solver: SolverMode,

    #[arg(short, long)]
    preprocess: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SolverMode {
    /// Use the internal solver
    Internal,
    /// Use the CaDiCaL solver
    Cadical,
}

impl Display for SolverMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SolverMode::Internal => "internal",
                SolverMode::Cadical => "cadical",
            }
        )
    }
}

/// The main entry point to Roole.
///
/// Takes one argument, a path to an SMT-LIB2 file.
/// Only the QF_BV logic is (partially) supported.
fn main() {
    let args = Args::parse();

    // open the file to be read
    let file = match std::fs::File::open(&args.input_file) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("File should be readable: {:?}", err);
            return;
        }
    };
    let reader = BufReader::new(file);

    // evaluate the file with the parser
    eprintln!("Evaluating file {:?}", args.input_file);
    parser::parse(
        reader,
        args.input_file,
        args.output_dir,
        args.solver,
        args.preprocess,
    );
    eprintln!("Finished evaluation");
}
