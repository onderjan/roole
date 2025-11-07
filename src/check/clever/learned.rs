use crate::check::{Assignment, clever::learned::bdd::Bdd};

mod bdd;

pub struct Learned {
    assignments: Vec<Assignment>,
    bdd: Bdd,
}

impl Learned {
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            bdd: Bdd::new(),
        }
    }

    pub fn number(&self) -> usize {
        self.assignments.len()
    }

    pub fn contains(&self, assignment: &Assignment) -> bool {
        self.assignments
            .iter()
            .any(|learned| learned.contains(assignment))
    }

    pub fn add(&mut self, assignment: &Assignment) {
        /*eprintln!(
            "Add zeros: {:#b}, ones: {:#b}, width: {:?}",
            zeros, ones, total_width
        );*/
        self.bdd.add(assignment);

        self.assignments.push(assignment.clone());

        //self.print();
    }

    pub fn print(&self) {
        self.bdd.print();
    }
}
