use std::{
    cmp,
    collections::BTreeMap,
    fmt::Write,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use roole::ExitValue;

use crate::exit_value::exit_value_str;

pub struct Stats {
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

impl Stats {
    pub fn new(num_files: usize, silent: bool) -> Self {
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

    pub fn inc_exit_value(&self, exit_value: Option<ExitValue>) {
        self.num_processed_files.fetch_add(1, Ordering::SeqCst);

        {
            // drop the guard before updating the progress bar
            // so it does not race
            let mut exit_value_numbers = self
                .exit_value_numbers
                .lock()
                .expect("Exit value numbers lock should not be poisoned");
            *exit_value_numbers
                .entry(OptionalExitValue(exit_value))
                .or_default() += 1;
        }

        self.update_progress_bar();
    }

    pub fn update_progress_bar(&self) {
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

    pub fn finish(&self) {
        let Some(progress_bar) = &self.progress_bar else {
            return;
        };

        self.update_progress_bar();
        progress_bar.finish();
    }
}
