use std::{fs::File, io::BufWriter};

use crate::{
    check::Assignment,
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        value::ThreeValued,
    },
};

pub struct Partition {
    nodes: Vec<Node>,

    current_node: Option<usize>,
    assignment: Assignment,
    decision_level: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct Decision {
    pub variable_index: usize,
    pub bit_index: u32,
    pub is_true: bool,
}

struct Node {
    parent: Option<usize>,
    decision: Decision,
    child_zero: Option<usize>,
    child_one: Option<usize>,
}

impl Partition {
    pub fn new(start_assignment: Assignment) -> Self {
        Self {
            assignment: start_assignment,
            nodes: Vec::new(),
            current_node: None,
            decision_level: 0,
        }
    }

    pub fn push_zero_decision(&mut self) {
        let Some(current_node) = self.current_node else {
            self.push_decision(Decision {
                variable_index: 0,
                bit_index: 0,
                is_true: false,
            });
            return;
        };

        let current_node = &mut self.nodes[current_node];
        let current_decision = &current_node.decision;

        let mut next_variable_index = current_decision.variable_index;
        let mut next_bit_index = current_decision.bit_index + 1;
        if next_bit_index
            >= self.assignment.values[current_decision.variable_index]
                .bound()
                .width()
        {
            next_bit_index = 0;
            next_variable_index += 1;
        }

        self.push_decision(Decision {
            variable_index: next_variable_index,
            bit_index: next_bit_index,
            is_true: false,
        });
    }

    fn push_decision(&mut self, decision: Decision) {
        let new_node_index = self.nodes.len();

        let new_node = Node {
            parent: self.current_node,
            decision,
            child_zero: None,
            child_one: None,
        };
        self.nodes.push(new_node);

        if let Some(current_node) = self.current_node {
            let current_node = &mut self.nodes[current_node];
            let child = if decision.is_true {
                &mut current_node.child_one
            } else {
                &mut current_node.child_zero
            };

            assert!(child.is_none());
            *child = Some(new_node_index);
        }

        self.current_node = Some(new_node_index);
        self.assignment.values[decision.variable_index]
            .set_bit_to_three_valued(decision.bit_index, ThreeValued::from_bool(decision.is_true));
        self.decision_level += 1;
    }

    pub fn inc_decision(&mut self) -> bool {
        while let Some(current_node) = self.current_node {
            let current_node = &mut self.nodes[current_node];
            let current_decision = &current_node.decision;

            if current_decision.is_true {
                // pop and go to the higher decision
                self.pop_decision();
            } else {
                // pop and push last decision true
                let new_decision = Decision {
                    variable_index: current_decision.variable_index,
                    bit_index: current_decision.bit_index,
                    is_true: true,
                };

                self.pop_decision();
                self.push_decision(new_decision);

                // we have incremented, return true
                return true;
            }
        }

        // increment overflowed
        false
    }

    fn pop_decision(&mut self) {
        let current_node = self
            .current_node
            .expect("Current node should be present when popping decision");
        let current_node = &mut self.nodes[current_node];
        let decision = &current_node.decision;

        // return to parent
        self.current_node = current_node.parent;
        self.assignment.values[decision.variable_index]
            .set_bit_to_three_valued(decision.bit_index, ThreeValued::Unknown);
        self.decision_level -= 1;
    }

    pub fn assignment(&self) -> &Assignment {
        &self.assignment
    }

    pub fn decision_level(&self) -> u64 {
        self.decision_level
    }

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;

        for (index, node) in self.nodes.iter().enumerate() {
            let label = format!(
                "{}{}.{}",
                if node.decision.is_true { "" } else { "!" },
                node.decision.variable_index,
                node.decision.bit_index
            );
            writeln!(f, "{} [label=\"{}\"]", index, label)?;
            if let Some(child_zero) = node.child_zero {
                writeln!(f, "{} -> {} [style=\"dashed\"]", index, child_zero)?;
            }
            if let Some(child_one) = node.child_one {
                writeln!(f, "{} -> {}", index, child_one)?;
            }
        }
        writeln!(f, "}}")?;
        Ok(())
    }

    pub fn write(&self) {
        let file = File::create("partition.dot").expect("Partition file should be created");
        self.write_dot(&mut BufWriter::new(file))
            .expect("Partition file should be written");
    }

    pub fn rev_decision_iter(&self) -> impl Iterator<Item = Decision> {
        struct RevDecisionIter<'a>(&'a Vec<Node>, Option<usize>);

        impl Iterator for RevDecisionIter<'_> {
            type Item = Decision;

            fn next(&mut self) -> Option<Self::Item> {
                match self.1 {
                    Some(current_node) => {
                        let current_node = &self.0[current_node];
                        self.1 = current_node.parent;
                        Some(current_node.decision)
                    }
                    None => None,
                }
            }
        }

        RevDecisionIter(&self.nodes, self.current_node)
    }
}
