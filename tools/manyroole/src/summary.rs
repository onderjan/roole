use std::{
    fs::File,
    io::Write,
    path::Path,
    process::ExitStatus,
    sync::mpsc::{self, Receiver, Sender},
};

pub struct Summary {
    file: File,
    sender: Sender<SummaryPart>,
    receiver: Receiver<SummaryPart>,
}

pub struct SummarySender {
    inner: Sender<SummaryPart>,
}

impl Summary {
    pub fn new(instance_name: Option<&String>, output_dir: &Path) -> Self {
        let summary_name = if let Some(instance_name) = &instance_name {
            format!("summary_{}.txt", instance_name)
        } else {
            String::from("summary.txt")
        };

        let file =
            File::create(output_dir.join(summary_name)).expect("Summary file should be created");

        let (sender, receiver) = mpsc::channel::<SummaryPart>();

        Self {
            file,
            sender,
            receiver,
        }
    }

    pub fn process(&mut self) {
        while let Ok(part) = self.receiver.try_recv() {
            part.write(&mut self.file);
        }
    }

    pub fn finish(mut self) {
        std::mem::drop(self.sender);
        for part in self.receiver.iter() {
            part.write(&mut self.file);
        }
    }

    pub fn sender(&self) -> SummarySender {
        SummarySender {
            inner: self.sender.clone(),
        }
    }
}

impl SummarySender {
    pub fn send(&self, file_name: String, status: ExitStatus, output_type: String) {
        self.inner
            .send(SummaryPart {
                file_name,
                status,
                output_type,
            })
            .expect("Should send summary");
    }
}

struct SummaryPart {
    file_name: String,
    status: ExitStatus,
    output_type: String,
}

impl SummaryPart {
    pub fn write(&self, summary_file: &mut File) {
        writeln!(
            summary_file,
            "{}; {}; {}",
            self.file_name, self.status, self.output_type
        )
        .expect("Summary file should be writable");
        summary_file
            .flush()
            .expect("Summary file should be flushable");
    }
}
