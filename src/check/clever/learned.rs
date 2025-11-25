use std::{fs::File, io::BufWriter};

use crate::check::{Assignment, clever::learned::rtree::RTree};

mod bdd;
mod rtree;

pub struct Learned {
    assignments: Vec<Assignment>,
    //bdd: Bdd,
    rtree: RTree,
}

impl Learned {
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            //bdd: Bdd::new(),
            rtree: RTree::new(),
        }
    }

    pub fn number(&self) -> usize {
        self.assignments.len()
    }

    pub fn contains(&self, assignment: &Assignment) -> bool {
        /*self.assignments
        .iter()
        .any(|learned| learned.contains(assignment))*/

        self.rtree.contains(assignment)
    }

    pub fn add(&mut self, assignment: &Assignment) {
        self.assignments.push(assignment.clone());

        /*eprintln!(
            "Add zeros: {:#b}, ones: {:#b}, width: {:?}",
            zeros, ones, total_width
        );*/
        //self.bdd.add(assignment);

        if !self.rtree.contains(assignment) {
            self.rtree.insert(assignment.clone());
        }

        //self.print();
    }

    pub fn write(&self) {
        //self.bdd.print();

        let learned_file = File::create("learned.dot").expect("Learned file should be created");
        self.rtree
            .write_dot(&mut BufWriter::new(learned_file))
            .expect("Learned file should be written");
    }
}
