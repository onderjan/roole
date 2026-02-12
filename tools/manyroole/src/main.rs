use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use walkdir::WalkDir;

struct ManyRoole {
    input_dir: PathBuf,
    output_dir: PathBuf,
    num_files: usize,
    num_processed_files: usize,
}

impl ManyRoole {
    fn new(input_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            input_dir,
            output_dir,
            num_files: 0,
            num_processed_files: 0,
        }
    }

    fn exec_roole(&self, path: &Path) -> std::process::Output {
        let mut command = Command::new("cargo");
        command.arg("run");
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

    fn process_smt2_file(&mut self, path: &Path) {
        let executed = self.exec_roole(path);

        let mut output_path = self.output_dir.clone().join(path);
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

        self.num_processed_files += 1;

        eprintln!(
            "[{}/{}] {}: {}",
            self.num_processed_files, self.num_files, input_dir_relative_path, executed.status
        );
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

    fn execute(mut self) {
        self.num_files = Self::iterate_smt2_files(self.input_dir.clone()).count();

        for entry in Self::iterate_smt2_files(self.input_dir.clone()) {
            self.process_smt2_file(entry.path());
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    dir: PathBuf,
    #[arg(long)]
    output_dir: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let output_dir = args.output_dir.unwrap_or(PathBuf::from("output/manyroole"));

    match std::fs::remove_dir_all(&output_dir) {
        Ok(_) => {}
        Err(err) => {
            if !matches!(err.kind(), std::io::ErrorKind::NotFound) {
                panic!("Output directory should be removable: {:?}", err);
            }
        }
    }

    let manyroole = ManyRoole::new(args.dir, output_dir);

    manyroole.execute();
}
