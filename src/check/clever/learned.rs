use crate::check::Assignment;

mod bdd;
mod linear;
mod rtree;

pub trait Learned {
    fn new() -> Self;

    fn contains(&self, assignment: &Assignment) -> bool;

    fn add(&mut self, assignment: &Assignment);

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()>;
}

#[allow(unused_imports)]
pub use {bdd::BddLearned, linear::LinearLearned, rtree::RTreeLearned};
