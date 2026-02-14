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
use roole::ExitValue;
use walkdir::WalkDir;

struct ManyRoole {
    input_dir: PathBuf,
    output_dir: PathBuf,
}

struct Stats {
    start_instant: Instant,
    num_files: usize,
    num_processed_files: AtomicUsize,
    progress_bar: indicatif::ProgressBar,
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
}

impl Stats {
    fn new(num_files: usize) -> Self {
        let start_instant = Instant::now();
        let progress_bar = indicatif::ProgressBar::new(num_files as u64);
        progress_bar.set_style(
            indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {percent}% {msg}",
            )
            .unwrap(),
        );
        Self {
            start_instant,
            num_files,
            num_processed_files: AtomicUsize::new(0),
            progress_bar,
            exit_value_numbers: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    fn update_progress_bar(&self) {
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

        self.progress_bar.set_position(num_processed_files as u64);
        self.progress_bar.set_message(message);
    }

    fn finish(&self) {
        self.update_progress_bar();
        self.progress_bar.finish();
    }
}

fn exit_value_str(exit_value: ExitValue) -> &'static str {
    match exit_value {
        ExitValue::Standard => "standard",
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
    fn new(input_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            input_dir,
            output_dir,
        }
    }

    fn exec_roole(&self, path: &Path) -> std::process::Output {
        let mut command = Command::new("cargo");
        command.arg("run");
        command.arg("--release");
        command.arg("--bin");
        command.arg("roole");
        command.arg("--");
        command.arg(path);
        command.arg("--solver");
        command.arg("none");
        command.arg("--preprocess");
        command.arg("--hexadecimal");

        command.output().expect("Cargo should execute")
    }

    fn process_smt2_file(&self, path: &Path, stats: &Stats, summary_sender: mpsc::Sender<Summary>) {
        let executed = self.exec_roole(path);

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

        let mut output_path = self.output_dir.clone().join(output_type).join(path);
        output_path.set_extension("out");

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

        let input_dir_relative_path = pathdiff::diff_paths(path, &self.input_dir)
            .expect("File path should be expressible relatively");
        let input_dir_relative_path = input_dir_relative_path
            .as_os_str()
            .to_str()
            .expect("Relative file path should be UTF-8");

        stats.num_processed_files.fetch_add(1, Ordering::SeqCst);

        summary_sender
            .send(Summary {
                file_name: input_dir_relative_path.to_string(),
                status: executed.status,
            })
            .expect("Should send summary");
    }

    fn iterate_smt2_files(dir: PathBuf) -> impl Iterator<Item = walkdir::DirEntry> {
        WalkDir::new(dir).into_iter().filter_map(|entry| {
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

    fn process_summary(summary: Summary, summary_file: &mut File) {
        writeln!(summary_file, "{}; {}", summary.file_name, summary.status)
            .expect("Summary file should be writeable");
    }

    fn execute(self) {
        match std::fs::remove_dir_all(&self.output_dir) {
            Ok(_) => {}
            Err(err) => {
                if !matches!(err.kind(), std::io::ErrorKind::NotFound) {
                    panic!("Output directory should be removable: {:?}", err);
                }
            }
        }

        std::fs::create_dir_all(&self.output_dir).expect("Output dir should be created");

        let mut summary_file = File::create(self.output_dir.join("summary.txt"))
            .expect("Summary file should be created");
        let (summary_sender, summary_receiver) = mpsc::channel::<Summary>();

        let input_dir = self.input_dir.clone();
        let num_files = Self::iterate_smt2_files(input_dir.clone()).count();

        let stats = Stats::new(num_files);

        let many_roole = Arc::new(self);
        let stats = Arc::new(stats);

        stats.update_progress_bar();

        {
            let thread_pool = rayon::ThreadPoolBuilder::new()
                .build()
                .expect("Thread pool should be built");

            for entry in Self::iterate_smt2_files(input_dir) {
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
struct Args {
    /// Directory where SMT-LIB2 files should be processed.
    dir: PathBuf,
    /// Directory in which the outputs will be put.
    #[arg(long)]
    output_dir: Option<PathBuf>,
    /// Number of bytes to which to limit memory allocation by each Roole instance.
    #[arg(long)]
    limit_alloc: Option<u64>,
}

fn main() {
    let args = Args::parse();

    let output_dir = args.output_dir.unwrap_or(PathBuf::from("output/manyroole"));

    let manyroole = ManyRoole::new(args.dir, output_dir);

    manyroole.execute();
}
