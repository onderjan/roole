use crate::{
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        value::ThreeValued,
    },
    problem::{Assignment, Decision},
};

use super::Learned;

#[derive(Debug)]
pub struct RooleLearned {
    nodes: Vec<Node>,
}

#[derive(Debug)]
enum Node {
    Inner(Vec<Child>),
    Value,
}

#[derive(Debug)]
struct Child {
    decision: Decision,
    phase: bool,
    index: usize,
}

impl Learned for RooleLearned {
    fn new() -> Self {
        Self {
            nodes: vec![Node::Inner(Vec::new())],
        }
    }

    fn contains(&self, assignment: &Assignment) -> bool {
        self.contains_recursive(assignment, 0)
    }

    fn add(&mut self, assignment: Assignment) {
        self.add_recursive(assignment, 0);
    }
    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;

        writeln!(f, "0 [label=\"Root\"]")?;

        for (index, node) in self.nodes.iter().enumerate() {
            match node {
                Node::Inner(children) => {
                    writeln!(f, "{} [shape=point, label=\"\"]", index)?;
                    for child in children {
                        writeln!(
                            f,
                            "{} -> {} [label=\"{}{:?}\"]",
                            index,
                            child.index,
                            if child.phase { "" } else { "!" },
                            child.decision
                        )?;
                    }
                }
                Node::Value => {
                    writeln!(f, "{} [label=\"\"]", index)?;
                }
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

impl RooleLearned {
    fn contains_recursive(&self, assignment: &Assignment, node_index: usize) -> bool {
        let children = match &self.nodes[node_index] {
            Node::Inner(children) => children,
            Node::Value => return true,
        };

        for child in children {
            let bit_value = assignment.get_decision_value(child.decision);

            let covered_by_decision = match bit_value {
                ThreeValued::False => !child.phase,
                ThreeValued::True => child.phase,
                ThreeValued::Unknown => false,
            };

            if covered_by_decision && self.contains_recursive(assignment, child.index) {
                return true;
            }
        }

        false
    }

    fn add_recursive(&mut self, mut assignment: Assignment, node_index: usize) {
        let num_nodes = self.nodes.len();

        let children = match &mut self.nodes[node_index] {
            Node::Inner(children) => children,
            Node::Value => {
                // already covered
                return;
            }
        };

        for child in children.iter_mut() {
            let bit_value = assignment.get_decision_value(child.decision);

            let covered_by_child = match bit_value {
                ThreeValued::False => !child.phase,
                ThreeValued::True => child.phase,
                ThreeValued::Unknown => false,
            };

            if covered_by_child {
                assignment.set_decision_value(child.decision, ThreeValued::Unknown);
                let child_index = child.index;
                self.add_recursive(assignment, child_index);
                return;
            }
        }

        // no child covers this
        // create a new child with a chosen decision

        for variable_index in 0..assignment.values().len() {
            for bit_index in 0..assignment.values()[variable_index].bound().width() {
                let decision = Decision::new(variable_index, bit_index);
                let bit_value = assignment.get_decision_value(decision);

                let phase = match bit_value {
                    ThreeValued::False => false,
                    ThreeValued::True => true,
                    ThreeValued::Unknown => continue,
                };

                let child_index = num_nodes;

                children.push(Child {
                    decision: Decision::new(variable_index, bit_index),
                    phase,
                    index: child_index,
                });

                self.nodes.push(Node::Inner(Vec::new()));

                assignment.set_decision_value(decision, ThreeValued::Unknown);
                self.add_recursive(assignment, child_index);

                return;
            }
        }

        // no decisions remain
        // change this to value node
        self.nodes[node_index] = Node::Value;
    }
}
