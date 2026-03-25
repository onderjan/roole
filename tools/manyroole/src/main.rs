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
    eprintln!("Finished");
}

fn process(args: ManyRooleArgs, stats: Arc<Stats>, summary: &mut Summary) {
    let temp_proof_dir = args.output_dir.join("temp_proofs");
    std::fs::create_dir_all(&temp_proof_dir).expect("Temporary proof directory should be created");
    let mut builder = rayon::ThreadPoolBuilder::new();
    if let Some(num_workers) = args.num_workers {
        builder = builder.num_threads(num_workers.try_into().unwrap());
    }
    let thread_pool = builder.build().expect("Thread pool should be built");
    eprintln!(
        "Number of threads in thread pool: {}",
        thread_pool.current_num_threads()
    );

    stats.update_progress_bar();
    for (index, entry) in iterate_smt2_paths(args.input_paths.as_slice()).enumerate() {
        summary.process();
        let path = entry.path().to_path_buf();
        let stats = Arc::clone(&stats);
        let sender = summary.sender();
        let args = args.clone();
        let temp_proof_output = temp_proof_dir.join(format!("proof_{}", index));
        thread_pool.spawn(move || {
            process_smt2_file(&args, &path, &stats, sender, &temp_proof_output);
        });
    }
}

fn process_smt2_file(
    args: &ManyRooleArgs,
    input_path: &Path,
    stats: &Stats,
    sender: SummarySender,
    temp_proof_output: &Path,
) {
    let roole_output = exec_roole(
        args.roole_binary.as_deref(),
        args.solver,
        input_path,
        temp_proof_output,
    );

    let status = roole_output.status;
    let exit_value = status.code().and_then(ExitValue::from_i32);

    let output_type = exit_value.map(exit_value_str).unwrap_or("other");
    let output_relative = compute_output_relative(args.input_root.as_deref(), input_path);
    let output_path = args.output_dir.join(output_type).join(output_relative);

    let roole_output_path = output_path.with_extension("out");
    let roole_proof_path = output_path.with_extension("proof");

    write_output(roole_output, &roole_output_path);
    if temp_proof_output.is_file() {
        std::fs::rename(temp_proof_output, roole_proof_path).expect("Proof file should be movable");
    }

    let path_str = input_path
        .as_os_str()
        .to_str()
        .expect("Relative file path should be UTF-8");

    sender.send(path_str.to_string(), status, output_type.to_string());
    stats.inc_exit_value(exit_value);
}
