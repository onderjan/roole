use std::{io::BufReader, path::PathBuf};

use clap::Parser;

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
    parser::parse(reader, args.input_file, args.output_dir);
    eprintln!("Finished evaluation");
}
