mod heap_limit;

pub fn init_resources() {
    heap_limit::init_heap_limit();
}
