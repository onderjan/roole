use std::fmt::Debug;

use crate::{
    domain::bitvector::{RBound, abstr::BitvectorDomain},
    problem::Assignment,
};

use super::Learned;

#[derive(Clone, Debug)]
pub struct LinearLearned<D: BitvectorDomain<Bound = RBound>>
where
    Assignment<D>: Debug,
{
    assignments: Vec<Assignment<D>>,
}

impl<D: BitvectorDomain<Bound = RBound>> Learned<D> for LinearLearned<D>
where
    Assignment<D>: Debug,
{
    fn new() -> Self {
        Self {
            assignments: Vec::new(),
        }
    }

    fn contains(&self, assignment: &Assignment<D>) -> bool {
        for learned in &self.assignments {
            if learned.contains(assignment) {
                return true;
            }
        }
        false
    }

    fn add(&mut self, assignment: Assignment<D>) {
        self.assignments.push(assignment);
    }

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;
        writeln!(f, "rankdir=\"LR\"")?;
        for (index, assignment) in self.assignments.iter().enumerate() {
            let label = format!("{:?}", assignment).replace("\"", "\\\"");
            writeln!(f, "{} [label=\"{}\"]", index, label)?;
        }
        writeln!(f, "}}")
    }
}
