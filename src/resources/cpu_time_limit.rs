use std::{
    env::VarError,
    sync::{
        Arc, LazyLock, Mutex,
        mpsc::{self, RecvTimeoutError, Sender},
    },
    thread::JoinHandle,
    time::Duration,
};

use cpu_time::ProcessTime;
use roole::ExitValue;

use crate::resources;

const ENV_VAR_NAME: &str = "ROOLE_CPU_TIME_LIMIT";

pub struct CpuTimeLimit {
    start_time: ProcessTime,
    limit: Option<TimeLimit>,
}

pub struct TimeLimit {
    time_limit: Duration,
    join_handle: JoinHandle<()>,
    finish_sender: Sender<()>,
}

static CPU_TIME_LIMIT: LazyLock<Arc<Mutex<Option<CpuTimeLimit>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

pub fn start() {
    let start_time = ProcessTime::try_now();

    let mut guard = CPU_TIME_LIMIT
        .lock()
        .expect("Lock should not be already held");

    assert!(guard.is_none());

    let Some(time_limit) = compute_cpu_time_limit() else {
        return;
    };

    let start_time = match start_time {
        Ok(ok) => ok,
        Err(err) => panic!("CPU time cannot be limited as it cannot be read: {:?}", err),
    };

    eprintln!("CPU time limited to {:?} ({})", time_limit, ENV_VAR_NAME);

    // start a thread that keeps checking the time limit
    let (finish_sender, finish_receiver) = mpsc::channel();

    let join_handle = std::thread::spawn(move || {
        loop {
            let remaining_duration = check(start_time, time_limit);
            match finish_receiver.recv_timeout(remaining_duration) {
                Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                    // received that we should finish monitoring or the other side disconnected, break
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {
                    // receive timeout, continue looping
                }
            }
        }
    });

    *guard = Some(CpuTimeLimit {
        start_time,
        limit: Some(TimeLimit {
            time_limit,
            finish_sender,
            join_handle,
        }),
    });
}

pub fn finish() {
    let mut guard = CPU_TIME_LIMIT
        .lock()
        .expect("Lock should not be already held");

    let Some(inner) = guard.as_mut() else {
        return;
    };

    let Some(limit) = inner.limit.take() else {
        // no limit
        return;
    };

    // make the monitoring thread finish
    // ignore if it cannot be done (worst case, we will wait for join forever)
    let _ = limit.finish_sender.send(());
    // join the monitoring thread
    if let Err(err) = limit.join_handle.join() {
        panic!("Could not join time-limit-keeping thread: {:?}", err)
    }
    // perform one final check that the limit was not exceeded
    check(inner.start_time, limit.time_limit);
}

pub fn print_used() {
    let guard = CPU_TIME_LIMIT
        .lock()
        .expect("Lock should not be already held");

    let Some(inner) = guard.as_ref() else {
        // no limit, do not print anything
        return;
    };

    let taken_duration = ProcessTime::now().duration_since(inner.start_time);
    eprintln!("Used CPU time: {:?}", taken_duration);
}

fn check(start_time: ProcessTime, time_limit: Duration) -> Duration {
    let current_time = ProcessTime::now();

    let remaining = current_time.duration_since(start_time);

    if let Some(remaining_duration) = time_limit.checked_sub(remaining)
        && !remaining_duration.is_zero()
    {
        // we still have some duration remaining, return it
        remaining_duration
    } else {
        // timeout, print resources, error message, and terminate
        resources::print_used();
        eprintln!("Time limit exceeded (set by {})", ENV_VAR_NAME);
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
