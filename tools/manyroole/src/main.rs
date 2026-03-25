use std::{
    fs::File,
    io::Write,
    path::Path,
    process::Command,
    sync::{Arc, mpsc},
};

use clap::Parser;
use num_traits::FromPrimitive;
use roole::{ExitValue, args::SolverMode};

use crate::{
    args::ManyRooleArgs,
    exit_value::exit_value_str,
    paths::{compute_output_path, iterate_smt2_paths},
    stats::Stats,
    summary::{Summary, process_summary},
};

mod args;
mod exit_value;
mod paths;
mod stats;
mod summary;

fn main() {
    let args = ManyRooleArgs::parse();

    if !args.retain_output_dir {
        match std::fs::remove_dir_all(&args.output_dir) {
            Ok(_) => {}
            Err(err) => {
                if !matches!(err.kind(), std::io::ErrorKind::NotFound) {
                    panic!("Output directory should be removable: {:?}", err);
                }
            }
        }
    }

    std::fs::create_dir_all(&args.output_dir).expect("Output dir should be created");

    let summary_name = if let Some(instance_name) = &args.instance_name {
        format!("summary_{}.txt", instance_name)
    } else {
        String::from("summary.txt")
    };

    let mut summary_file =
        File::create(args.output_dir.join(summary_name)).expect("Summary file should be created");
    let (summary_sender, summary_receiver) = mpsc::channel::<Summary>();

    let num_files = iterate_smt2_paths(args.input_paths.as_slice()).count();

    let stats = Stats::new(num_files, args.silent);
    let stats = Arc::new(stats);

    stats.update_progress_bar();

    {
        let mut builder = rayon::ThreadPoolBuilder::new();
        if let Some(num_workers) = args.num_workers {
            builder = builder.num_threads(num_workers.try_into().unwrap());
        }
        let thread_pool = builder.build().expect("Thread pool should be built");

        for entry in iterate_smt2_paths(args.input_paths.as_slice()) {
            while let Ok(summary) = summary_receiver.try_recv() {
                process_summary(summary, &mut summary_file);
            }
            stats.update_progress_bar();
            let path = entry.path().to_path_buf();
            let stats = Arc::clone(&stats);
            let summary_sender = summary_sender.clone();
            let args = args.clone();
            thread_pool.install(|| {
                thread_pool.spawn(move || {
                    process_smt2_file(&args, &path, &stats, summary_sender);
                });
            });
        }

        std::mem::drop(summary_sender);
    }

    // no thread pool anymore

    for summary in summary_receiver.iter() {
        process_summary(summary, &mut summary_file);
    }

    stats.finish();
}

fn process_smt2_file(
    args: &ManyRooleArgs,
    input_path: &Path,
    stats: &Stats,
    summary_sender: mpsc::Sender<Summary>,
) {
    let executed = exec_roole(args.roole_binary.as_deref(), args.solver, input_path);

    let exit_value = executed.status.code().and_then(ExitValue::from_i32);
    stats.inc_exit_value(exit_value);

    let output_type = exit_value.map(exit_value_str).unwrap_or("other");
    let output_path = compute_output_path(args.input_root.as_deref(), input_path);
    let output_path = args.output_dir.join(output_type).join(output_path);

    let output_parent_dir = output_path
        .parent()
        .expect("Output file should have a parent");
    std::fs::create_dir_all(output_parent_dir).expect("Output parent dirs should be created");

    let mut file = File::create(output_path).expect("Output file should be created");

    let stdout = String::from_utf8(executed.stdout).expect("Stdout should be UTF-8");
    let stderr = String::from_utf8(executed.stderr).expect("Stderr should be UTF-8");

    writeln!(
        file,
        "Exit status: {}\n\n=== STDOUT ===\n\n{}\n\n=== STDERR ===\n\n{}",
        executed.status, stdout, stderr
    )
    .expect("Output file should be writable");

    stats.inc_num_processed_files();

    let path_str = input_path
        .as_os_str()
        .to_str()
        .expect("Relative file path should be UTF-8");

    summary_sender
        .send(Summary {
            file_name: path_str.to_string(),
            status: executed.status,
            output_type: output_type.to_string(),
        })
        .expect("Should send summary");
}

fn exec_roole(binary: Option<&Path>, solver: SolverMode, path: &Path) -> std::process::Output {
    let mut command = if let Some(roole) = binary {
        Command::new(roole)
    } else {
        let mut command = Command::new("cargo");
        command.arg("run");
        command.arg("--release");
        command.arg("--bin");
        command.arg("roole");
        if matches!(solver, SolverMode::Cadical) {
            command.arg("--features");
            command.arg("cadical");
        }
        command.arg("--");
        command
    };
    command.arg(path);
    command.arg("--solver");
    command.arg(solver.to_string());
    command.arg("--preprocess");
    command.arg("--hexadecimal");

    command.output().expect("Cargo should execute")
}
