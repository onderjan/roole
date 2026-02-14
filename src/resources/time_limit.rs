use std::{
    env::VarError,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use crate::exit::ExitValue;

const ENV_VAR_NAME: &str = "ROOLE_TIME_LIMIT";

pub struct TimeLimit(Option<SetTimeLimit>);

pub struct SetTimeLimit {
    limit_instant: Instant,
    join_handle: JoinHandle<()>,
    should_finish: Arc<AtomicBool>,
}

#[must_use]
pub fn start() -> TimeLimit {
    let start_time = Instant::now();
    let Some(time_limit) = compute_time_limit() else {
        return TimeLimit(None);
    };

    let limit_instant = start_time
        .checked_add(time_limit)
        .expect("Limit time instant should be representable");

    eprintln!(
        "Execution time limited to {:?} ({})",
        time_limit, ENV_VAR_NAME
    );

    // start a thread that keeps checking the time limit

    let should_finish = Arc::new(AtomicBool::new(false));
    let should_finish_thread = Arc::clone(&should_finish);

    let join_handle = std::thread::spawn(move || {
        while !should_finish_thread.load(Ordering::SeqCst) {
            let sleep_duration = limit_instant.duration_since(Instant::now());
            std::thread::sleep(sleep_duration);
            check_time_limit(limit_instant);
        }
    });

    TimeLimit(Some(SetTimeLimit {
        limit_instant,
        should_finish,
        join_handle,
    }))
}

impl TimeLimit {
    pub fn finish(self) {
        let Some(inner) = self.0 else {
            return;
        };
        inner.should_finish.store(true, Ordering::SeqCst);
        if let Err(err) = inner.join_handle.join() {
            panic!("Could not join time-limit-keeping thread: {:?}", err)
        }
        check_time_limit(inner.limit_instant);
    }
}

fn check_time_limit(limit_instant: Instant) {
    let finish_time = Instant::now();

    if finish_time > limit_instant {
        // we timeouted
        eprintln!("Time limit exceeded (set by {})", ENV_VAR_NAME);
        // immediately end with timeout code
        std::process::exit(ExitValue::TimeLimitExceeded as i32);
    }
}

fn compute_time_limit() -> Option<Duration> {
    let mut value = match std::env::var(ENV_VAR_NAME) {
        Ok(value) => value,
        Err(VarError::NotUnicode(_)) => {
            panic!("{} must be Unicode", ENV_VAR_NAME)
        }
        Err(VarError::NotPresent) => {
            // no limit
            return None;
        }
    };

    let Some(unit_prefix) = value.pop() else {
        // consider empty variable unset, no limit
        return None;
    };

    const SECONDS_IN_MINUTE: u64 = 60;
    const MINUTES_IN_HOUR: u64 = 60;
    const HOURS_IN_DAY: u64 = 24;

    const SECONDS_IN_HOUR: u64 = SECONDS_IN_MINUTE * MINUTES_IN_HOUR;
    const SECONDS_IN_DAY: u64 = SECONDS_IN_HOUR * HOURS_IN_DAY;

    let seconds_multiplier = match unit_prefix {
        's' => 1,
        'm' => SECONDS_IN_MINUTE,
        'h' => SECONDS_IN_HOUR,
        'd' => SECONDS_IN_DAY,
        _ => {
            // return the character back
            value.push(unit_prefix);
            1
        }
    };

    let Ok(value) = value.parse::<u64>() else {
        panic!(
            "{} must be an unsigned number with optional postfix s/m/h/d",
            ENV_VAR_NAME
        );
    };

    let Some(seconds_limit) = value.checked_mul(seconds_multiplier) else {
        panic!("{} must have number of seconds fit in u64", ENV_VAR_NAME);
    };

    Some(Duration::from_secs(seconds_limit))
}
