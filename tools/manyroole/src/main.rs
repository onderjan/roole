use crate::args::ManyRooleArgs;
use clap::Parser;

mod args;
mod exit_value;
mod manyroole;
mod stats;

fn main() {
    let args = ManyRooleArgs::parse();
    manyroole::execute(args);
}
