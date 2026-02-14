use clap::Parser;
use roole::{ExitValue, args::Args};

mod resources;

/// The main entry point to Roole.
fn main() -> ExitValue {
    // initialise resources
    let resources = resources::init();

    let args = Args::parse();
    let exit = ExitValue::from_roole_result(roole::exec(args));

    // ensure that the resources have not been exceeded
    resources.finish();

    exit
}
