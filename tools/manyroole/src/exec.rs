use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use roole::args::SolverMode;

#[derive(Clone)]
pub struct RunlimArgs {
    pub runlim_time_limit: Option<u64>,
    pub runlim_space_limit: Option<u64>,
}

pub fn exec_roole(
    roole_binary: &Path,
    runlim: Option<RunlimArgs>,
    solver: SolverMode,
    problem_file: &Path,
    output_dir: &Path,
    output_name: String,
    preprocess: bool,
) -> std::process::ExitStatus {
    let output_name = format!("{}.roole", output_name);

    let mut command = ExecCommand::new(
        roole_binary,
        runlim,
        output_dir.to_path_buf(),
        output_name.clone(),
    );
    command.arg(problem_file);
    command.arg("--solver");
    command.arg(solver.to_string());
    command.arg("--hexadecimal");
    command.arg("--proof-output");
    command.arg(output_dir.join(format!("{}.proof", output_name)));

    if preprocess {
        command.arg("--preprocess");
    }

    command.exec()
}

pub fn exec_roolean(
    roolean_binary: &Path,
    runlim: Option<RunlimArgs>,
    problem_file: &Path,
    proof_file: &Path,
    output_dir: &Path,
    output_name: String,
) -> std::process::ExitStatus {
    let output_name = format!("{}.roolean", output_name);
    let mut command = ExecCommand::new(
        roolean_binary,
        runlim,
        output_dir.to_path_buf(),
        output_name.clone(),
    );
    command.arg(problem_file);
    command.arg(proof_file);

    command.exec()
}

struct ExecCommand {
    command: Command,
    output_dir: PathBuf,
    output_name: String,
}

impl ExecCommand {
    fn new(
        binary: &Path,
        runlim: Option<RunlimArgs>,
        output_dir: PathBuf,
        output_name: String,
    ) -> Self {
        let command = if let Some(runlim) = runlim {
            let mut command = Command::new("runlim");
            command.arg("-o");
            command.arg(output_dir.join(format!("{}.runlim", output_name)));
            command.arg("--propagate");
            command.arg("--single");
            if let Some(time_limit) = runlim.runlim_time_limit {
                command.arg(format!("--time-limit={}", time_limit));
            }
            if let Some(space_limit) = runlim.runlim_space_limit {
                command.arg(format!("--space-limit={}", space_limit));
            }

            command.arg(binary);

            command
        } else {
            Command::new(binary)
        };

        Self {
            command,
            output_dir,
            output_name,
        }
    }

    fn arg<S: AsRef<OsStr>>(&mut self, arg: S) {
        self.command.arg(arg);
    }

    fn exec(mut self) -> std::process::ExitStatus {
        let output = self.command.output().expect("Command should execute");
        std::fs::write(
            self.output_dir.join(format!("{}.stderr", self.output_name)),
            output.stderr,
        )
        .expect("Stderr should be written");

        output.status
    }
}
