use std::{
    cmp,
    collections::BTreeMap,
    fmt::Write as _,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

use clap::Parser;
use num_traits::FromPrimitive;
use roole::{
    ExitValue,
    args::{DEFAULT_SOLVER_MODE, SolverMode},
};
use walkdir::WalkDir;

struct ManyRoole {
    args: ManyRooleArgs,
}

struct Stats {
    start_instant: Instant,
    num_files: usize,
    num_processed_files: AtomicUsize,
    progress_bar: Option<indicatif::ProgressBar>,
    exit_value_numbers: Arc<Mutex<BTreeMap<OptionalExitValue, u64>>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct OptionalExitValue(Option<ExitValue>);

impl PartialOrd for OptionalExitValue {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OptionalExitValue {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // make None compare higher
        match (self.0, other.0) {
            (None, None) => cmp::Ordering::Equal,
            (None, Some(_)) => cmp::Ordering::Greater,
            (Some(_), None) => cmp::Ordering::Less,
            (Some(a), Some(b)) => a.cmp(&b),
        }
    }
}

struct Summary {
    file_name: String,
    status: ExitStatus,
    output_type: String,
}

impl Stats {
    fn new(num_files: usize, silent: bool) -> Self {
        let start_instant = Instant::now();

        let progress_bar = if silent {
            None
        } else {
            let progress_bar = indicatif::ProgressBar::new(num_files as u64);
            progress_bar.set_style(
                indicatif::ProgressStyle::with_template(
                    "[{elapsed_precise}] {bar:40.cyan/blue} {percent}% {msg}",
                )
                .unwrap(),
            );
            Some(progress_bar)
        };
        Self {
            start_instant,
            num_files,
            num_processed_files: AtomicUsize::new(0),
            progress_bar,
            exit_value_numbers: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn update_progress_bar(&self) {
        let Some(progress_bar) = &self.progress_bar else {
            return;
        };

        let num_processed_files = self.num_processed_files.load(Ordering::SeqCst);
        let current_instant = Instant::now();

        let elapsed = current_instant
            .checked_duration_since(self.start_instant)
            .unwrap_or(Duration::ZERO);

        // total estimated time: elapsed + remaining
        // for 1 file: elapsed / num_processed_files
        // for remaining files = elapsed * (num_remaining_files / num_processed_files)

        let num_remaining_files = self.num_files - num_processed_files;

        let remaining_ratio = num_remaining_files as f64 / num_processed_files as f64;

        let remaining_seconds = elapsed.as_secs_f64() * remaining_ratio;

        let completion_msg = if remaining_seconds.is_finite() && remaining_seconds >= 0. {
            let remaining = Duration::from_secs_f64(remaining_seconds).as_secs();

            let hours = remaining / 3600;
            let minutes = (remaining / 60) % 60;
            let seconds = remaining % 60;

            let mut msg = format!(" ({:0>2}:{:0>2}:{:0>2} remaining)", hours, minutes, seconds);

            let exit_value_numbers = self
                .exit_value_numbers
                .lock()
                .expect("Exit value numbers lock should not be poisoned");

            let mut first = true;

            if !exit_value_numbers.is_empty() {
                let _ = write!(msg, ": ",);
            }

            for (exit_value, number) in exit_value_numbers.iter() {
                if first {
                    first = false;
                } else {
                    let _ = write!(msg, ", ");
                }

                let _ = write!(
                    msg,
                    "{} {}",
                    number,
                    exit_value.0.map(exit_value_str).unwrap_or("other")
                );
            }

            msg
        } else {
            String::new()
        };

        let message = format!(
            "{}/{}{}",
            num_processed_files, self.num_files, completion_msg
        );

        progress_bar.set_position(num_processed_files as u64);
        progress_bar.set_message(message);
    }

    fn finish(&self) {
        let Some(progress_bar) = &self.progress_bar else {
            return;
        };

        self.update_progress_bar();
        progress_bar.finish();
    }
}

fn exit_value_str(exit_value: ExitValue) -> &'static str {
    match exit_value {
        ExitValue::Success => "success",
        ExitValue::Satisfiable => "sat",
        ExitValue::WrongSatisfiable => "wrong_sat",
        ExitValue::Unsatisfiable => "unsat",
        ExitValue::WrongUnsatisfiable => "wrong_unsat",
        ExitValue::Unknown => "unknown",
        ExitValue::TimeLimitExceeded => "time_limit",
        ExitValue::HeapLimitExceeded => "heap_limit",
        ExitValue::Panic => "panic",
    }
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

        {
            let mut exit_value_numbers = stats
                .exit_value_numbers
                .lock()
                .expect("Exit value numbers lock should not be poisoned");
            *exit_value_numbers
                .entry(OptionalExitValue(exit_value))
                .or_default() += 1;
        }

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

        stats.num_processed_files.fetch_add(1, Ordering::SeqCst);

        /*let input_dir_relative_path = pathdiff::diff_paths(path, &self.args.input_dir)
            .expect("File path should be expressible relatively");
        let input_dir_relative_path = input_dir_relative_path
            .as_os_str()
            .to_str()
            .expect("Relative file path should be UTF-8");*/
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

    fn execute(self) {
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ManyRooleArgs {
    /// Directory in which the outputs will be put.
    #[arg(long, default_value = "output/manyroole")]
    output_dir: PathBuf,

    /// Whether to retain the output directory or delete it beforehand.
    #[arg(long)]
    retain_output_dir: bool,

    /// The solver to use.
    #[arg(long, default_value_t = DEFAULT_SOLVER_MODE)]
    solver: SolverMode,

    /// Name of the instance for summary printing.
    #[arg(long)]
    instance_name: Option<String>,

    /// Input directory that will be taken as a root of the input paths for writing output files.
    #[arg(long)]
    input_root: Option<PathBuf>,

    /// SMT-LIB2 files or directories to process.
    #[structopt(required = true)]
    input_paths: Vec<PathBuf>,

    /// The Roole binary to use. If not set, `cargo run --package roole` will be used.
    #[arg(long)]
    roole_binary: Option<PathBuf>,

    #[arg(long)]
    silent: bool,

    /// Number of worker threads that will be used for parallel computation.
    #[arg(long)]
    num_workers: Option<u32>,
}

fn main() {
    let args = ManyRooleArgs::parse();

    ManyRoole { args }.execute();
}
