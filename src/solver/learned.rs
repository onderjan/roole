#![allow(dead_code)]

use crate::problem::assignment::Assignment;

pub mod bdd;
pub mod linear;
pub mod roole;
pub mod rtree;

pub trait Learned {
    fn new() -> Self;
    fn contains(&self, assignment: &Assignment) -> bool;
    fn add(&mut self, assignment: Assignment);

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()>;
}
