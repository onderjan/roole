use std::fmt::Debug;

/// A decision on an assignment variable and its bit.
#[derive(Clone, Copy)]
pub struct Decision {
    variable_index: usize,
    bit_index: u32,
}

impl Decision {
    pub fn new(variable_index: usize, bit_index: u32) -> Self {
        Self {
            variable_index,
            bit_index,
        }
    }

    pub fn variable_index(&self) -> usize {
        self.variable_index
    }

    pub fn bit_index(&self) -> u32 {
        self.bit_index
    }
}

impl Debug for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.variable_index, self.bit_index)
    }
}
