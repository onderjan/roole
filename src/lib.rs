use std::io::BufReader;

use crate::{args::Args, exit::RooleResult, solver::SolverSettings};

mod domain;
mod parser;
mod problem;
mod solver;

pub mod args;
mod exit;

pub use exit::ExitValue;

pub fn exec(args: Args) -> RooleResult {
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
        debug_dir: args.debug_dir,
        proof_output: args.proof_output,
        solver_mode: args.solver,
    };

    let roole_result = parser::parse(reader, args.input_file, settings);

    eprintln!("Finished evaluation");

    roole_result
}
