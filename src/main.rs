use std::io::BufReader;

mod check;
mod domain;
mod formula;
mod parse;

fn main() {
    let mut args = std::env::args();

    // skip over the zeroth argument which gives the called executable
    args.next();

    let Some(path) = args.next() else {
        eprintln!("Expected exactly one argument");
        return;
    };

    if args.next().is_some() {
        eprintln!("Expected exactly one argument");
        return;
    }

    let file = match std::fs::File::open(&path) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("File should be readable: {:?}", err);
            return;
        }
    };
    let reader = BufReader::new(file);

    eprintln!("Evaluating file {}", path);

    parse::evaluate(reader, Some(path));

    eprintln!("Finished evaluation");
}
