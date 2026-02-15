mod cpu_time_limit;
mod heap_limit;

#[must_use]
pub fn init() -> Resources {
    cpu_time_limit::start();
    heap_limit::init();

    Resources(())
}

pub struct Resources(());

impl Resources {
    pub fn finish(self) {
        cpu_time_limit::finish();
        heap_limit::finish();

        print_used();
    }
}

fn print_used() {
    eprintln!("----------");
    cpu_time_limit::print_used();
    heap_limit::print_used();
    eprintln!("----------");
}
