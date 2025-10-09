use std::io::BufReader;

use aws_smt_ir::{CommandStream, smt2parser::concrete};

mod bitvector;

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

    let stream = CommandStream::new(reader, concrete::SyntaxBuilder, Some(path));
    let commands = stream
        .collect::<Result<Vec<_>, _>>()
        .expect("File should be SMT-LIB-2 parseable");

    for command in commands {
        println!("{:#?}", command);
        match command {
            concrete::Command::Assert { term } => {
                println!("TODO: assert");
            }
            concrete::Command::CheckSat => {
                println!("TODO: check-sat");
            }
            concrete::Command::DeclareFun {
                symbol,
                parameters,
                sort,
            } => {
                println!("TODO: declare-fun")
            }
            concrete::Command::Exit => {
                break;
            }
            concrete::Command::SetInfo { keyword, value } => {
                // ignore
            }
            concrete::Command::SetLogic { symbol } => {
                assert_eq!(symbol.0, "QF_BV");
            }
            _ => {
                eprintln!("Command not supported: {:?}", command);
                return;
            }
        }
    }
}
