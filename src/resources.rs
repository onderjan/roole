use crate::resources::cpu_time_limit::CpuTimeLimit;

mod cpu_time_limit;
mod heap_limit;

#[must_use]
pub fn init() -> Resources {
    let time_limit = cpu_time_limit::start();
    heap_limit::init_heap_limit();

    Resources { time_limit }
}

pub struct Resources {
    time_limit: CpuTimeLimit,
}

impl Resources {
    pub fn finish(self) {
        self.time_limit.finish();
    }
}
