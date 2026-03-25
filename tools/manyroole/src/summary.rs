use std::{fs::File, io::Write, process::ExitStatus};

pub struct Summary {
    pub file_name: String,
    pub status: ExitStatus,
    pub output_type: String,
}

pub fn process_summary(summary: Summary, summary_file: &mut File) {
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
