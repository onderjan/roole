use std::{path::Path, sync::Arc};

use clap::Parser;
use num_traits::FromPrimitive;
use roole::ExitValue;

use crate::{
    args::ManyRooleArgs,
    exec::{RunlimArgs, exec_roole, exec_roolean},
    exit_value::exit_value_str,
    paths::{compute_output_relative, iterate_smt2_paths, remove_output_dir},
    stats::Stats,
    summary::{Summary, SummarySender},
};

mod args;
mod build_roolean;
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
    let roole_binary = args
        .roole_binary
        .clone()
        .unwrap_or_else(|| build_roolean::build(args.solver));

    let temp_dir = args.output_dir.join("temp");
    std::fs::create_dir_all(&temp_dir).expect("Temporary proof directory should be created");
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
        let problem_file = entry.path().to_path_buf();
        let stats = Arc::clone(&stats);
        let sender = summary.sender();
        let args = args.clone();
        let roole_binary = roole_binary.clone();
        let temp_output_dir = temp_dir.join(format!(
            "temp_{}_{}",
            args.instance_name.clone().unwrap_or_default(),
            index
        ));
        thread_pool.spawn(move || {
            process_smt2_file(
                &args,
                &roole_binary,
                &problem_file,
                &stats,
                sender,
                &temp_output_dir,
            );
        });
    }
}

fn process_smt2_file(
    args: &ManyRooleArgs,
    roole_binary: &Path,
    problem_file: &Path,
    stats: &Stats,
    sender: SummarySender,
    temp_output_dir: &Path,
) {
    let problem_stem = problem_file
        .file_stem()
        .expect("Problem file should have a stem")
        .to_string_lossy()
        .to_string();
    std::fs::create_dir_all(temp_output_dir).expect("Temporary output directory should be created");

    let runlim = if args.runlim {
        Some(RunlimArgs {
            runlim_time_limit: args.runlim_time_limit,
            runlim_space_limit: args.runlim_space_limit,
        })
    } else {
        None
    };

    let roole_status = exec_roole(
        roole_binary,
        runlim.clone(),
        args.solver,
        problem_file,
        temp_output_dir,
        problem_stem.clone(),
        args.preprocess,
    );

    let mut solved = false;

    let mut output_kind = if let Some(code) = roole_status.code() {
        if let Some(exit_value) = ExitValue::from_i32(code) {
            if matches!(
                exit_value,
                ExitValue::Satisfiable | ExitValue::Unsatisfiable
            ) {
                solved = true;
            }
            exit_value_str(exit_value).to_string()
        } else {
            match code {
                2 => String::from("runlim_time"),
                3 => String::from("runlim_memory"),
                _ => format!("other_{}", code),
            }
        }
    } else {
        String::from("other")
    };

    let roolean_status = if let Some(roolean_binary) = &args.roolean_binary
        && solved
    {
        // proof-check
        let roolean_status = exec_roolean(
            roolean_binary,
            runlim,
            problem_file,
            &temp_output_dir.join(format!("{}.roole.proof", problem_stem)),
            temp_output_dir,
            problem_stem,
        );
        if roolean_status.success() {
            output_kind += "_proven";
        } else if let Some(code) = roolean_status.code() {
            let code_value = match code {
                2 => String::from("runlim_time"),
                3 => String::from("runlim_memory"),
                101 => String::from("error"),
                _ => format!("other_{}", code),
            };
            output_kind = format!("{}_unproven_{}", output_kind, code_value);
        } else {
            output_kind += "_unproven";
        }
        Some(roolean_status)
    } else {
        None
    };

    let final_relative = compute_output_relative(args.input_root.as_deref(), problem_file);
    let final_relative_dir = final_relative
        .parent()
        .expect("Relative file should have a parent");
    let final_dir = args.output_dir.join(&output_kind).join(final_relative_dir);

    std::fs::create_dir_all(&final_dir).expect("Final output directory should be created");

    for file in std::fs::read_dir(temp_output_dir).expect("Temporary output dir should be readable")
    {
        let file = file.expect("Temporary dir entry should be readable");

        let from = file.path();
        let to = final_dir.join(file.file_name());

        std::fs::rename(from, to).expect("File should be movable from temporary to final dir");
    }

    std::fs::remove_dir(temp_output_dir).expect("Temporary output dir should be removable");

    let path_str = problem_file
        .as_os_str()
        .to_str()
        .expect("Relative file path should be UTF-8");

    sender.send(
        path_str.to_string(),
        roole_status,
        roolean_status,
        output_kind.clone(),
    );
    stats.inc_kind(output_kind);
}
