#![allow(dead_code)]

pub mod bdd;
pub mod linear;
pub mod rtree;

pub trait Learned {
    fn new() -> Self;
    fn contains(&self, assignment: &Assignment) -> bool;
    fn add(&mut self, assignment: &Assignment);

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()>;
}

use crate::assignment::Assignment;
