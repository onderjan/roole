use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::{Arc, mpsc},
};

use num_traits::FromPrimitive;
use roole::{ExitValue, args::SolverMode};
use walkdir::WalkDir;

use crate::{args::ManyRooleArgs, exit_value::exit_value_str, stats::Stats};

pub fn execute(args: ManyRooleArgs) {
    ManyRoole { args }.execute();
}

pub struct ManyRoole {
    args: ManyRooleArgs,
}

struct Summary {
    file_name: String,
    status: ExitStatus,
    output_type: String,
}

impl ManyRoole {
    fn exec_roole(&self, path: &Path) -> std::process::Output {
        let mut command = if let Some(roole) = &self.args.roole_binary {
            Command::new(roole)
        } else {
            let mut command = Command::new("cargo");
            command.arg("run");
            command.arg("--release");
            command.arg("--bin");
            command.arg("roole");
            if matches!(self.args.solver, SolverMode::Cadical) {
                command.arg("--features");
                command.arg("cadical");
            }
            command.arg("--");
            command
        };
        command.arg(path);
        command.arg("--solver");
        command.arg(self.args.solver.to_string());
        command.arg("--preprocess");
        command.arg("--hexadecimal");

        command.output().expect("Cargo should execute")
    }

    fn compute_output_path(input_root: Option<&Path>, input_path: &Path) -> PathBuf {
        let input_root = input_root
            .map(Path::to_path_buf)
            .unwrap_or_else(|| std::env::current_dir().expect("Current directory should be valid"));

        let Ok(input_root) = std::path::absolute(&input_root) else {
            panic!(
                "Input root should be expressible as absolute: {:?}",
                input_root
            );
        };

        let Ok(input_path) = std::path::absolute(input_path) else {
            panic!("Path should be expressible as absolute: {:?}", input_path);
        };

        // conver the path relative to input root
        let Some(mut output_path) = pathdiff::diff_paths(&input_path, &input_root) else {
            panic!(
                "Path {:?} should be expressible relative to input root {:?}",
                input_path, input_root
            );
        };

        if output_path.is_absolute() {
            panic!("Path should be relative-expressible: {:?}", output_path);
        }

        if output_path.iter().any(|a| a == "..") {
            panic!(
                "Path expressed relative to input root should not contain '..': {:?}",
                output_path
            );
        }

        output_path.set_extension("out");

        output_path
    }

    fn process_smt2_file(
        &self,
        input_path: &Path,
        stats: &Stats,
        summary_sender: mpsc::Sender<Summary>,
    ) {
        let executed = self.exec_roole(input_path);

        let exit_value = executed.status.code().and_then(ExitValue::from_i32);
        stats.inc_exit_value(exit_value);

        let output_type = exit_value.map(exit_value_str).unwrap_or("other");
        let output_path = Self::compute_output_path(self.args.input_root.as_deref(), input_path);
        let output_path = self.args.output_dir.join(output_type).join(output_path);

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

    fn iterate_smt2_path(path: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
        WalkDir::new(path).into_iter().filter_map(|entry| {
            let entry = match entry {
                Ok(ok) => ok,
                Err(error) => panic!("Error walking directory: {:?}", error),
            };

            if entry.path().extension()? == "smt2" {
                Some(entry)
            } else {
                None
            }
        })
    }

    fn iterate_smt2_paths(paths: &[PathBuf]) -> Box<dyn Iterator<Item = walkdir::DirEntry> + '_> {
        let mut iterator: Box<dyn Iterator<Item = walkdir::DirEntry>> =
            Box::new(std::iter::empty());
        for path in paths {
            iterator = Box::new(iterator.chain(Self::iterate_smt2_path(path)));
        }
        iterator
    }

    fn process_summary(summary: Summary, summary_file: &mut File) {
        writeln!(
            summary_file,
            "{}; {}; {}",
            summary.file_name, summary.status, summary.output_type
        )
        .expect("Summary file should be writable");
        summary_file
            .flush()
            .expect("Summary file should be flushable");
    }

    pub fn execute(self) {
        if !self.args.retain_output_dir {
            match std::fs::remove_dir_all(&self.args.output_dir) {
                Ok(_) => {}
                Err(err) => {
                    if !matches!(err.kind(), std::io::ErrorKind::NotFound) {
                        panic!("Output directory should be removable: {:?}", err);
                    }
                }
            }
        }

        std::fs::create_dir_all(&self.args.output_dir).expect("Output dir should be created");

        let summary_name = if let Some(instance_name) = &self.args.instance_name {
            format!("summary_{}.txt", instance_name)
        } else {
            String::from("summary.txt")
        };

        let mut summary_file = File::create(self.args.output_dir.join(summary_name))
            .expect("Summary file should be created");
        let (summary_sender, summary_receiver) = mpsc::channel::<Summary>();

        let num_files = Self::iterate_smt2_paths(self.args.input_paths.as_slice()).count();

        let stats = Stats::new(num_files, self.args.silent);

        let many_roole = Arc::new(self);
        let stats = Arc::new(stats);

        stats.update_progress_bar();

        {
            let mut builder = rayon::ThreadPoolBuilder::new();
            if let Some(num_workers) = many_roole.args.num_workers {
                builder = builder.num_threads(num_workers.try_into().unwrap());
            }
            let thread_pool = builder.build().expect("Thread pool should be built");

            for entry in Self::iterate_smt2_paths(many_roole.args.input_paths.as_slice()) {
                while let Ok(summary) = summary_receiver.try_recv() {
                    Self::process_summary(summary, &mut summary_file);
                }
                stats.update_progress_bar();
                let path = entry.path().to_path_buf();
                let many_roole = Arc::clone(&many_roole);
                let stats = Arc::clone(&stats);
                let summary_sender = summary_sender.clone();
                thread_pool.install(|| {
                    thread_pool.spawn(move || {
                        many_roole.process_smt2_file(&path, &stats, summary_sender);
                    });
                });
            }

            std::mem::drop(summary_sender);
        }

        // no thread pool anymore

        for summary in summary_receiver.iter() {
            Self::process_summary(summary, &mut summary_file);
        }

        stats.finish();
    }
}
