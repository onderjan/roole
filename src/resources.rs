use crate::resources::time_limit::TimeLimit;

mod heap_limit;
mod time_limit;

#[must_use]
pub fn init() -> Resources {
    let time_limit = time_limit::start();
    heap_limit::init_heap_limit();

    Resources { time_limit }
}

pub struct Resources {
    time_limit: TimeLimit,
}

impl Resources {
    pub fn finish(self) {
        self.time_limit.finish();
    }
}
