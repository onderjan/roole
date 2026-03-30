use std::{path::Path, sync::Arc};

use clap::Parser;
use num_traits::FromPrimitive;
use roole::ExitValue;

use crate::{
    args::ManyRooleArgs,
    exec::{exec_roole, exec_roolean},
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
        let temp_output_dir = temp_dir.join(format!(
            "temp_{}_{}",
            args.instance_name.clone().unwrap_or_default(),
            index
        ));
        thread_pool.spawn(move || {
            process_smt2_file(&args, &problem_file, &stats, sender, &temp_output_dir);
        });
    }
}

fn process_smt2_file(
    args: &ManyRooleArgs,
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

    let roole_status = exec_roole(
        args.roole_binary.as_deref(),
        args.time,
        args.solver,
        problem_file,
        temp_output_dir,
        problem_stem.clone(),
        args.preprocess,
    );

    let roole_exit_value = roole_status.code().and_then(ExitValue::from_i32);

    let mut output_kind = roole_exit_value
        .map(exit_value_str)
        .unwrap_or("other")
        .to_string();

    let solved = matches!(
        roole_exit_value,
        Some(ExitValue::Satisfiable) | Some(ExitValue::Unsatisfiable)
    );

    let roolean_status = if let Some(roolean_binary) = &args.roolean_binary
        && solved
    {
        // proof-check
        let roolean_status = exec_roolean(
            roolean_binary,
            args.time,
            problem_file,
            &temp_output_dir.join(format!("{}.roole.proof", problem_stem)),
            temp_output_dir,
            problem_stem,
        );
        if roolean_status.success() {
            output_kind += "_proven";
        } else {
            output_kind += "_unproven";
        }
        Some(roolean_status)
    } else {
        None
    };

    /*
    let output_relative = compute_output_relative(args.input_root.as_deref(), problem_file);
    let output_path = args.output_dir.join(&output_kind).join(output_relative);

    let roole_output_path = output_path.with_extension("out");
    let roole_proof_path = output_path.with_extension("proof");

    //write_output(roole_output, &roole_output_path);
    if let Some(roolean_output) = roolean_output {
        let roolean_output_path = output_path.with_extension("roolean.out");
        write_output(roolean_output, &roolean_output_path);
    }
    */

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
