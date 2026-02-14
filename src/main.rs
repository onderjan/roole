use std::{fmt::Display, io::BufReader, path::PathBuf};

use clap::{Parser, ValueEnum};

use crate::{exit::ExitValue, solver::SolverSettings};

mod domain;
mod exit;
mod parser;
mod problem;
mod resources;
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

    #[arg(short = 'H', long)]
    hexadecimal: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SolverMode {
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

/// The main entry point to Roole.
///
/// Takes one argument, a path to an SMT-LIB2 file.
/// Only the QF_BV logic is (partially) supported.
fn main() -> ExitValue {
    let resources = resources::init();

    let args = Args::parse();

    // open the file to be read
    let file = match std::fs::File::open(&args.input_file) {
        Ok(ok) => ok,
        Err(err) => {
            panic!("File should be readable: {:?}", err);
        }
    };
    let reader = BufReader::new(file);

    // evaluate the file with the parser
    eprintln!("Evaluating file {:?}", args.input_file);
    let settings = SolverSettings {
        output_dir: args.output_dir,
        solver_mode: args.solver,
        preprocess: args.preprocess,
        hexadecimal: args.hexadecimal,
    };

    let parser_result = parser::parse(reader, args.input_file, settings);
    let exit = ExitValue::from_parser_result(parser_result);

    eprintln!("Finished evaluation");

    // ensure that the resources have not been exceeded
    resources.finish();

    exit
}
