use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use cargo_metadata::Message;

pub fn build() -> PathBuf {
    let mut command = Command::new("cargo");
    command.arg("build");
    command.arg("--message-format=json-render-diagnostics");
    command.arg("--release");
    command.arg("--bin");
    command.arg("roole");
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn().expect("Cargo build should be spawned");
    let reader =
        std::io::BufReader::new(child.stdout.take().expect("Build stdout should be taken"));

    let mut roole_binary = None;

    for message in Message::parse_stream(reader) {
        if let Message::CompilerArtifact(artifact) = message.expect("Message should be parsed")
            && let Some(executable) = artifact.executable
        {
            if roole_binary.is_none() {
                roole_binary = Some(executable.as_std_path().to_path_buf());
            } else {
                panic!("Expected no more than one Roole executable");
            }
        }
    }

    child.wait().expect("Cargo build should exit");

    roole_binary.expect("Roole binary should be built")
}
