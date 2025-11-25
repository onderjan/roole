use crate::check::Assignment;

pub struct Partitions {
    pub inner: Vec<Partition>,
}

pub struct Partition {
    assignment: Assignment,
    child_zero: Option<usize>,
    child_one: Option<usize>,
}

impl Partitions {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
}
