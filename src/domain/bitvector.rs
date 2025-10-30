pub mod abstr;
pub mod concr;

mod bound;

pub use bound::{BitvectorBound, CBound, RBound, compute_u64_mask};
