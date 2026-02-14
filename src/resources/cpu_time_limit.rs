use std::{
    env::VarError,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use cpu_time::ProcessTime;
use roole::ExitValue;

const ENV_VAR_NAME: &str = "ROOLE_CPU_TIME_LIMIT";

pub struct CpuTimeLimit(Option<TimeLimit>);

pub struct TimeLimit {
    start_time: ProcessTime,
    time_limit: Duration,
    join_handle: JoinHandle<()>,
    should_finish: Arc<AtomicBool>,
}

#[must_use]
pub fn start() -> CpuTimeLimit {
    let start_time = ProcessTime::try_now();

    let Some(time_limit) = compute_cpu_time_limit() else {
        return CpuTimeLimit(None);
    };

    let start_time = match start_time {
        Ok(ok) => ok,
        Err(err) => panic!("CPU time cannot be limited as it cannot be read: {:?}", err),
    };

    eprintln!("CPU time limited to {:?} ({})", time_limit, ENV_VAR_NAME);

    // start a thread that keeps checking the time limit

    let should_finish = Arc::new(AtomicBool::new(false));
    let should_finish_thread = Arc::clone(&should_finish);

    let join_handle = std::thread::spawn(move || {
        while !should_finish_thread.load(Ordering::SeqCst) {
            let remaining_duration = check(start_time, time_limit);
            std::thread::sleep(remaining_duration);
        }
    });

    CpuTimeLimit(Some(TimeLimit {
        start_time,
        time_limit,
        should_finish,
        join_handle,
    }))
}

impl CpuTimeLimit {
    pub fn finish(self) {
        let Some(inner) = self.0 else {
            return;
        };
        inner.should_finish.store(true, Ordering::SeqCst);
        if let Err(err) = inner.join_handle.join() {
            panic!("Could not join time-limit-keeping thread: {:?}", err)
        }
        // perform one final check that the limit was not exceeded
        let _ = check(inner.start_time, inner.time_limit);
    }
}

fn check(start_time: ProcessTime, time_limit: Duration) -> Duration {
    let current_time = ProcessTime::now();

    let remaining = current_time.duration_since(start_time);

    if let Some(remaining_duration) = time_limit.checked_sub(remaining) {
        // we still have some duration remaining, return it
        remaining_duration
    } else {
        // timeout
        eprintln!("Time limit exceeded (set by {})", ENV_VAR_NAME);
        // immediately end with timeout code
        std::process::exit(ExitValue::TimeLimitExceeded as i32);
    }
}

fn compute_cpu_time_limit() -> Option<Duration> {
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
