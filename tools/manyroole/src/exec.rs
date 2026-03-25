use std::{fs::File, io::Write, path::Path, process::Command};

use roole::args::SolverMode;

pub fn exec_roole(binary: Option<&Path>, solver: SolverMode, path: &Path) -> std::process::Output {
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

pub fn write_output(output: std::process::Output, path: &Path) {
    let output_parent_dir = path.parent().expect("Output file should have a parent");
    std::fs::create_dir_all(output_parent_dir).expect("Output parent dirs should be created");

    let mut file = File::create(path).expect("Output file should be created");

    let stdout = String::from_utf8(output.stdout).expect("Stdout should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("Stderr should be UTF-8");

    writeln!(
        file,
        "Exit status: {}\n\n=== STDOUT ===\n\n{}\n\n=== STDERR ===\n\n{}",
        output.status, stdout, stderr
    )
    .expect("Output file should be writable");
}
