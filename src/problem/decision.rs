use std::fmt::Debug;

#[derive(Clone, Copy)]
pub struct Decision {
    pub variable_index: usize,
    pub bit_index: u32,
}

impl Debug for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.variable_index, self.bit_index)
    }
}
