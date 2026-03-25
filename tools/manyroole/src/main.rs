use std::{path::Path, sync::Arc};

use clap::Parser;
use num_traits::FromPrimitive;
use roole::ExitValue;

use crate::{
    args::ManyRooleArgs,
    exec::{exec_roole, write_output},
    exit_value::exit_value_str,
    paths::{compute_output_relative, iterate_smt2_paths, remove_output_dir},
    stats::Stats,
    summary::{Summary, SummarySender},
};

mod args;
mod exec;
mod exit_value;
mod paths;
mod stats;
mod summary;

fn main() {
    let args = ManyRooleArgs::parse();

    if !args.retain_output_dir {
        remove_output_dir(&args.output_dir);
    }

    std::fs::create_dir_all(&args.output_dir).expect("Output dir should be created");

    let num_files = iterate_smt2_paths(args.input_paths.as_slice()).count();
    let stats = Arc::new(Stats::new(num_files, args.silent));
    let mut summary = Summary::new(args.instance_name.as_ref(), &args.output_dir);

    process(args, Arc::clone(&stats), &mut summary);

    // no thread pool anymore
    summary.finish();
    stats.finish();
}

fn process(args: ManyRooleArgs, stats: Arc<Stats>, summary: &mut Summary) {
    let mut builder = rayon::ThreadPoolBuilder::new();
    if let Some(num_workers) = args.num_workers {
        builder = builder.num_threads(num_workers.try_into().unwrap());
    }
    let thread_pool = builder.build().expect("Thread pool should be built");

    for entry in iterate_smt2_paths(args.input_paths.as_slice()) {
        summary.process();
        stats.update_progress_bar();
        let path = entry.path().to_path_buf();
        let stats = Arc::clone(&stats);
        let sender = summary.sender();
        let args = args.clone();
        thread_pool.spawn(move || {
            process_smt2_file(&args, &path, &stats, sender);
        });
    }
}

fn process_smt2_file(
    args: &ManyRooleArgs,
    input_path: &Path,
    stats: &Stats,
    summary_sender: SummarySender,
) {
    let roole_output = exec_roole(args.roole_binary.as_deref(), args.solver, input_path);

    let status = roole_output.status;
    let exit_value = status.code().and_then(ExitValue::from_i32);
    stats.inc_exit_value(exit_value);

    let output_type = exit_value.map(exit_value_str).unwrap_or("other");
    let output_relative = compute_output_relative(args.input_root.as_deref(), input_path);

    let mut roole_output_path = args.output_dir.join(output_type).join(output_relative);
    roole_output_path.set_extension("out");

    write_output(roole_output, &roole_output_path);

    stats.inc_num_processed_files();

    let path_str = input_path
        .as_os_str()
        .to_str()
        .expect("Relative file path should be UTF-8");

    summary_sender.send(path_str.to_string(), status, output_type.to_string())
}
