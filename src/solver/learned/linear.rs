use crate::{problem::Assignment, solver::Learned};

#[derive(Clone, Debug)]
pub struct LinearLearned {
    assignments: Vec<Assignment>,
}

impl Learned for LinearLearned {
    fn new() -> Self {
        Self {
            assignments: Vec::new(),
        }
    }

    fn contains(&self, assignment: &Assignment) -> bool {
        for learned in &self.assignments {
            if learned.contains(assignment) {
                return true;
            }
        }
        false
    }

    fn add(&mut self, assignment: Assignment) {
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
