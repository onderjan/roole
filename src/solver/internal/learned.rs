#![allow(dead_code)]

use crate::{
    domain::bitvector::{RBound, abstr::BitvectorDomain},
    problem::Assignment,
};

pub mod bdd;
pub mod linear;
pub mod roole;
pub mod rtree;

pub trait Learned<D: BitvectorDomain<Bound = RBound>> {
    fn new() -> Self;
    fn contains(&self, assignment: &Assignment<D>) -> bool;
    fn add(&mut self, assignment: Assignment<D>);

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()>;
}
