use std::io::BufReader;

mod domain;
mod parser;
mod problem;
mod solver;

/// The main entry point to Roole.
///
/// Takes one argument, a path to an SMT-LIB2 file.
/// Only the QF_BV logic is (partially) supported.
fn main() {
    let mut args = std::env::args();

    // skip over the zeroth argument which gives the called executable
    args.next();

    // get the first argument
    let Some(path) = args.next() else {
        eprintln!("Expected exactly one argument");
        return;
    };
    // ensure there are no other arguments
    if args.next().is_some() {
        eprintln!("Expected exactly one argument");
        return;
    }

    // open the file to be read
    let file = match std::fs::File::open(&path) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("File should be readable: {:?}", err);
            return;
        }
    };
    let reader = BufReader::new(file);

    // evaluate the file with the parser
    eprintln!("Evaluating file {}", path);
    parser::parse(reader, Some(path));
    eprintln!("Finished evaluation");
}
