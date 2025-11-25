use std::{fs::File, io::BufWriter};

use crate::{
    check::Assignment,
    domain::{
        bitvector::{BitvectorBound, abstr::BitvectorDomain},
        value::ThreeValued,
    },
};

#[derive(Debug)]
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
}

#[derive(Debug)]
struct Node {
    parent: Option<usize>,
    ty: NodeType,
}

#[derive(Debug)]
enum NodeType {
    Absent,
    NonLeaf(NonLeaf),
    Value(Value),
}

#[derive(Debug)]
struct NonLeaf {
    decision: Decision,
    child_zero: usize,
    child_one: usize,
}

#[derive(Debug)]
struct Value {
    inner: bool,
    novel: bool,
}

impl Partition {
    pub fn new(start_assignment: Assignment) -> Self {
        Self {
            assignment: start_assignment,
            nodes: vec![Node {
                parent: None,
                ty: NodeType::Absent,
            }],
            current_node: Some(0),
            decision_level: 0,
        }
    }

    pub fn set_current_value(&mut self, value: bool, novel: bool) {
        let current_node =
            &mut self.nodes[self.current_node.expect("Current node should be present")];

        assert!(matches!(current_node.ty, NodeType::Absent));

        current_node.ty = NodeType::Value(Value {
            inner: value,
            novel,
        });
    }

    pub fn choose_decision(&mut self) {
        //eprintln!("Choosing decision: {:?}", self);

        let current_node =
            &mut self.nodes[self.current_node.expect("Current node should be present")];

        let Some(parent_node) = current_node.parent else {
            self.make_nonleaf(Decision {
                variable_index: 0,
                bit_index: 0,
            });
            self.push_decision(false);
            return;
        };

        let NodeType::NonLeaf(parent) = &self.nodes[parent_node].ty else {
            panic!("Parent should be non-leaf");
        };
        let parent_decision = parent.decision;

        let mut next_variable_index = parent_decision.variable_index;
        let mut next_bit_index = parent_decision.bit_index + 1;
        if next_bit_index
            >= self.assignment.values[parent_decision.variable_index]
                .bound()
                .width()
        {
            next_bit_index = 0;
            next_variable_index += 1;
        }

        self.make_nonleaf(Decision {
            variable_index: next_variable_index,
            bit_index: next_bit_index,
        });

        self.push_decision(false);

        //eprintln!("After choosing decision: {:?}", self);
    }

    fn make_nonleaf(&mut self, decision: Decision) {
        let current_node = self.current_node.expect("Current node should be present");
        let child_zero = self.nodes.len();
        self.nodes.push(Node {
            parent: Some(current_node),
            ty: NodeType::Absent,
        });
        let child_one = self.nodes.len();
        self.nodes.push(Node {
            parent: Some(current_node),
            ty: NodeType::Absent,
        });

        let current_node = &mut self.nodes[current_node];
        assert!(matches!(current_node.ty, NodeType::Absent));

        current_node.ty = NodeType::NonLeaf(NonLeaf {
            decision,
            child_zero,
            child_one,
        });

        /*let new_node_index = self.nodes.len();

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
        */
    }

    fn push_decision(&mut self, selected_one: bool) {
        let current_node = self.current_node.expect("Current node should be present");

        let NodeType::NonLeaf(current_node) = &self.nodes[current_node].ty else {
            panic!("Node type should be non-leaf")
        };
        let decision = &current_node.decision;

        if selected_one {
            // go to child one
            self.current_node = Some(current_node.child_one);
            self.assignment.values[decision.variable_index]
                .set_bit_to_three_valued(decision.bit_index, ThreeValued::True);
        } else {
            // go to child zero
            self.current_node = Some(current_node.child_zero);
            self.assignment.values[decision.variable_index]
                .set_bit_to_three_valued(decision.bit_index, ThreeValued::False);
        }
        self.decision_level += 1;
    }

    pub fn inc_decision(&mut self) -> bool {
        loop {
            let current_node_index = self.current_node.expect("Current node should be present");
            let current_node = &self.nodes[current_node_index];

            let Some(parent_node) = current_node.parent else {
                // increment overflowed
                self.current_node = None;
                return false;
            };
            let parent_node = &self.nodes[parent_node];
            let NodeType::NonLeaf(parent_nonleaf) = &parent_node.ty else {
                panic!("Parent should be non-leaf");
            };

            if parent_nonleaf.child_one == current_node_index {
                // we are in the one (true) child
                // pop and go to the higher decision
                self.pop_decision();
                continue;
            }

            // we are in the zero (false) child
            // pop and push last decision true
            self.pop_decision();
            self.push_decision(true);

            // we have incremented, return true
            return true;
        }
    }

    fn pop_decision(&mut self) {
        let current_node = self.current_node.expect("Current node should be present");
        let current_node = &self.nodes[current_node];
        let parent_node = current_node
            .parent
            .expect("Current node should have parent");

        let NodeType::NonLeaf(parent_nonleaf) = &self.nodes[parent_node].ty else {
            panic!("Parent should be non-leaf")
        };
        let decision = &parent_nonleaf.decision;

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
            match &node.ty {
                NodeType::Absent => {
                    writeln!(f, "{} [label=\"-\"]", index)?;
                }
                NodeType::NonLeaf(non_leaf) => {
                    let label = format!(
                        "{}.{}",
                        non_leaf.decision.variable_index, non_leaf.decision.bit_index
                    );
                    writeln!(f, "{} [label=\"{}\"]", index, label)?;
                    writeln!(f, "{} -> {} [style=\"dashed\"]", index, non_leaf.child_zero)?;
                    writeln!(f, "{} -> {}", index, non_leaf.child_one)?;
                }
                NodeType::Value(value) => {
                    let label = format!("{}{}", value.inner, if value.novel { " (!)" } else { "" });
                    writeln!(f, "{} [label=\"{}\"]", index, label)?;
                }
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
                loop {
                    match self.1 {
                        Some(current_node) => {
                            let current_node = &self.0[current_node];
                            self.1 = current_node.parent;
                            let NodeType::NonLeaf(current_node) = &current_node.ty else {
                                continue;
                            };
                            return Some(current_node.decision);
                        }
                        None => return None,
                    }
                }
            }
        }

        RevDecisionIter(&self.nodes, self.current_node)
    }
}
