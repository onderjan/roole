use std::io::BufReader;

use aws_smt_ir::{CommandStream, smt2parser::concrete};

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
            concrete::Command::CheckSatAssuming { literals } => todo!(),
            concrete::Command::DeclareConst { symbol, sort } => todo!(),
            concrete::Command::DeclareDatatype { symbol, datatype } => todo!(),
            concrete::Command::DeclareDatatypes { datatypes } => todo!(),
            concrete::Command::DeclareFun {
                symbol,
                parameters,
                sort,
            } => {
                println!("TODO: declare-fun")
            }
            concrete::Command::DeclareSort { symbol, arity } => todo!(),
            concrete::Command::DefineFun { sig, term } => todo!(),
            concrete::Command::DefineFunRec { sig, term } => todo!(),
            concrete::Command::DefineFunsRec { funs } => todo!(),
            concrete::Command::DefineSort {
                symbol,
                parameters,
                sort,
            } => todo!(),
            concrete::Command::Echo { message } => todo!(),
            concrete::Command::Exit => {
                break;
            }
            concrete::Command::GetAssertions => todo!(),
            concrete::Command::GetAssignment => todo!(),
            concrete::Command::GetInfo { flag } => todo!(),
            concrete::Command::GetModel => todo!(),
            concrete::Command::GetOption { keyword } => todo!(),
            concrete::Command::GetProof => todo!(),
            concrete::Command::GetUnsatAssumptions => todo!(),
            concrete::Command::GetUnsatCore => todo!(),
            concrete::Command::GetValue { terms } => todo!(),
            concrete::Command::Pop { level } => todo!(),
            concrete::Command::Push { level } => todo!(),
            concrete::Command::Reset => todo!(),
            concrete::Command::ResetAssertions => todo!(),
            concrete::Command::SetInfo { keyword, value } => {
                // ignore
            }
            concrete::Command::SetLogic { symbol } => {
                assert_eq!(symbol.0, "BV");
            }
            concrete::Command::SetOption { keyword, value } => todo!(),
        }
    }
}
