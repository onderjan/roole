use std::{fmt::Debug, fs::File, io::BufWriter};

use crate::{
    domain::value::ThreeValued,
    problem::{
        Assignment, Decision, Problem,
        solution::{Proof, ProofDecisionNode, ProofNode},
    },
};

#[derive(Debug)]
pub struct Partition {
    nodes: Vec<Node>,
    decision_order: Vec<Decision>,

    current_node: Option<usize>,
    assignment: Assignment,
    decision_level: u64,
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
    ty: ValueType,
}

#[derive(Debug)]
pub enum ValueType {
    Normal,
    Learned,
    Backtracked,
}

impl Partition {
    pub fn new(problem: &Problem) -> Self {
        // fully unknown assignment at the start
        let assignment = problem.unknown_assignment();
        let mut decision_order = Vec::new();
        for (variable_index, width) in problem.variable_widths().iter().cloned().enumerate() {
            for bit_index in 0..width {
                decision_order.push(Decision::new(variable_index, bit_index));
            }
        }

        Self {
            assignment,
            decision_order,
            nodes: vec![Node {
                parent: None,
                ty: NodeType::Absent,
            }],
            current_node: Some(0),
            decision_level: 0,
        }
    }

    pub fn current_value(&self) -> Option<bool> {
        let current_node = self.current_node?;

        match &self.nodes[current_node].ty {
            NodeType::Absent => None,
            NodeType::NonLeaf(_) => None,
            NodeType::Value(value) => Some(value.inner),
        }
    }

    pub fn set_current_value(&mut self, value: bool, ty: ValueType) {
        let current_node =
            &mut self.nodes[self.current_node.expect("Current node should be present")];

        assert!(matches!(current_node.ty, NodeType::Absent));

        current_node.ty = NodeType::Value(Value { inner: value, ty });
    }

    pub fn choose_decision(&mut self, phase: bool) {
        let decision = self.decision_order[self.decision_level as usize];
        self.make_nonleaf(decision);
        self.push_decision(phase);
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
    }

    pub fn push_decision(&mut self, phase: bool) {
        let current_node = self.current_node.expect("Current node should be present");

        let NodeType::NonLeaf(current_node) = &self.nodes[current_node].ty else {
            panic!("Node type should be non-leaf")
        };
        let decision = current_node.decision;

        let (child, value) = match phase {
            true => (current_node.child_one, ThreeValued::True),
            false => (current_node.child_zero, ThreeValued::False),
        };

        self.current_node = Some(child);
        self.assignment.set_decision_value(decision, value);

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

    pub fn pop_decision(&mut self) {
        let current_node = self.current_node.expect("Current node should be present");
        let current_node = &self.nodes[current_node];
        let parent_node = current_node
            .parent
            .expect("Current node should have parent");

        let NodeType::NonLeaf(parent_nonleaf) = &self.nodes[parent_node].ty else {
            panic!("Parent should be non-leaf")
        };
        let decision = parent_nonleaf.decision;

        // return to parent
        self.current_node = current_node.parent;
        self.assignment
            .set_decision_value(decision, ThreeValued::Unknown);
        self.decision_level -= 1;
    }

    pub fn force_next_decision(&mut self, force_level: u64, known_phase: bool, known_value: bool) {
        let next_decision_index = self.decision_level;
        let force_decision_index = force_level - 1;

        assert_ne!(next_decision_index, force_decision_index);
        self.decision_order
            .swap(next_decision_index as usize, force_decision_index as usize);

        // we need to make the current node absent
        let current_node_index = self.current_node.expect("Current node should be present");
        let current_node = &mut self.nodes[current_node_index];
        current_node.ty = NodeType::Absent;

        // choose the decision, add backtracked

        let decision = self.decision_order[self.decision_level as usize];
        self.make_nonleaf(decision);
        let NodeType::NonLeaf(current_nonleaf) = &mut self.nodes[current_node_index].ty else {
            panic!("Current node should be nonleaf");
        };
        let known_child = match known_phase {
            false => current_nonleaf.child_zero,
            true => current_nonleaf.child_one,
        };

        self.nodes[known_child].ty = NodeType::Value(Value {
            inner: known_value,
            ty: ValueType::Backtracked,
        });

        // resolve the other phase
        self.push_decision(!known_phase);
    }

    pub fn assignment(&self) -> &Assignment {
        &self.assignment
    }

    pub fn decision_level(&self) -> u64 {
        self.decision_level
    }

    pub fn into_proof(self) -> Proof {
        let mut proof_nodes = Vec::new();

        for node in self.nodes {
            let proof_node = match node.ty {
                NodeType::Absent => ProofNode::Value(ThreeValued::Unknown),
                NodeType::NonLeaf(non_leaf) => ProofNode::Decision(ProofDecisionNode {
                    decision: non_leaf.decision,
                    child_zero: non_leaf.child_zero,
                    child_one: non_leaf.child_one,
                }),
                NodeType::Value(value) => ProofNode::Value(ThreeValued::from_bool(value.inner)),
            };

            proof_nodes.push(proof_node);
        }

        Proof::new(proof_nodes)
    }

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;

        for (index, node) in self.nodes.iter().enumerate() {
            match &node.ty {
                NodeType::Absent => {
                    writeln!(f, "{} [label=\"-\"]", index)?;
                }
                NodeType::NonLeaf(non_leaf) => {
                    let label = format!("{:?}", non_leaf.decision);
                    writeln!(f, "{} [label=\"{}\"]", index, label)?;
                    writeln!(f, "{} -> {} [style=\"dashed\"]", index, non_leaf.child_zero)?;
                    writeln!(f, "{} -> {}", index, non_leaf.child_one)?;
                }
                NodeType::Value(value) => {
                    let type_label = match value.ty {
                        ValueType::Normal => "",
                        ValueType::Learned => " (L)",
                        ValueType::Backtracked => " (B)",
                    };
                    writeln!(f, "{} [label=\"{}{}\"]", index, value.inner, type_label)?;
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

    pub fn rev_decision_iter(&self) -> impl Iterator<Item = (Decision, bool, bool)> {
        RevDecisionIter(&self.nodes, self.current_node, None)
    }
}

struct RevDecisionIter<'a>(&'a Vec<Node>, Option<usize>, Option<bool>);

impl Iterator for RevDecisionIter<'_> {
    type Item = (Decision, bool, bool);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // remember node and update it to parent
            let current_node_index = self.1?;
            let current_node = &self.0[current_node_index];
            self.1 = current_node.parent;

            // remember and update phase based on whether this is zero child or one child
            let current_phase = self.2;
            self.2 = if let Some(parent) = current_node.parent {
                let NodeType::NonLeaf(parent) = &self.0[parent].ty else {
                    panic!("Parent should be non-leaf")
                };

                Some(current_node_index == parent.child_one)
            } else {
                None
            };

            //
            let NodeType::NonLeaf(current_node) = &current_node.ty else {
                // not a non-leaf node, try parent next
                continue;
            };

            let phase_true = current_phase.expect("Non-leaf decision phase should be present");

            let backtracked_fn = |nodes: &Vec<Node>, child: usize| {
                matches!(
                    nodes[child].ty,
                    NodeType::Value(Value {
                        ty: ValueType::Backtracked,
                        ..
                    })
                )
            };

            let uses_backtracking = backtracked_fn(self.0, current_node.child_zero)
                || backtracked_fn(self.0, current_node.child_one);

            return Some((current_node.decision, phase_true, uses_backtracking));
        }
    }
}
