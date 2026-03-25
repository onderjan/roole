use std::collections::VecDeque;

use crate::{
    domain::{
        bitvector::{
            BitvectorBound, RBound,
            abstr::{BitvectorDomain, RBitvector, three_valued::ThreeValuedBitvector},
            concr::ConcreteBitvector,
        },
        value::ThreeValued,
    },
    problem::{Assignment, Decision, Evaluator, Problem},
};

/// Proof on the satisfiability problem.
#[derive(Debug)]
pub struct Proof {
    nodes: Vec<ProofNode>,
}

/// Proof node.
#[derive(Debug)]
pub enum ProofNode {
    /// Non-leaf decision.
    Decision(ProofDecisionNode),
    /// Leaf claimed evaluation value.
    Value(ThreeValued),
}

/// Proof decision node.
///
/// The decision splits a given variable bit
/// to the zero (false) case and one (true) case.
///
/// The two children are also proof nodes. Each one
/// must have its index greater than the index of
/// its parent, so the proof is guaranteed non-circular.
#[derive(Debug)]
pub struct ProofDecisionNode {
    pub decision: Decision,
    pub child_zero: usize,
    pub child_one: usize,
}

impl Proof {
    pub fn new(nodes: Vec<ProofNode>) -> Self {
        Self { nodes }
    }

    pub fn from_satisfying(assignment: &Assignment<ThreeValuedBitvector<RBound>>) -> Proof {
        let mut nodes = Vec::new();
        for (variable_index, var_value) in assignment.values().iter().enumerate() {
            for bit_index in 0..var_value.bound().width() {
                let decision = Decision::new(variable_index, bit_index);
                match var_value.three_valued_from_bit(bit_index).to_opt_bool() {
                    Some(known) => {
                        // split to the value not taken, which will have an unknown leaf,
                        // and the value taken
                        let not_taken = nodes.len() + 1;
                        let taken = not_taken + 1;
                        let (child_zero, child_one) = if known {
                            (not_taken, taken)
                        } else {
                            (taken, not_taken)
                        };

                        // push split node
                        nodes.push(ProofNode::Decision(ProofDecisionNode {
                            decision,
                            child_zero,
                            child_one,
                        }));
                        // push not taken node
                        nodes.push(ProofNode::Value(ThreeValued::Unknown));
                    }
                    None => {
                        // no splitting necessary
                    }
                }
            }
        }

        // add a leaf that makes the problem SAT at the end
        nodes.push(ProofNode::Value(ThreeValued::True));

        Proof::new(nodes)
    }

    pub fn write_smt(
        &self,
        mut f: impl std::io::Write,
        result: bool,
    ) -> Result<(), std::io::Error> {
        writeln!(&mut f, "(roole-proof {} ", result)?;
        enum StackValue {
            Node(usize),
            CloseParen,
        }
        let mut queue = VecDeque::from_iter([StackValue::Node(0)]);
        // we can pretty-print if we want to
        fn tab_to_column(mut f: impl std::io::Write, column: u32) -> Result<(), std::io::Error> {
            for _ in 0..column {
                write!(f, "\t")?;
            }
            Ok(())
        }
        let mut column = 1;
        while let Some(value) = queue.pop_front() {
            match value {
                StackValue::Node(node_index) => match &self.nodes[node_index] {
                    ProofNode::Decision(node) => {
                        tab_to_column(&mut f, column)?;
                        writeln!(
                            f,
                            "(decision {} {} ",
                            node.decision.variable_index(),
                            node.decision.bit_index()
                        )?;
                        column += 1;
                        queue.push_front(StackValue::CloseParen);
                        queue.push_front(StackValue::Node(node.child_one));
                        queue.push_front(StackValue::Node(node.child_zero));
                    }
                    ProofNode::Value(value) => {
                        tab_to_column(&mut f, column)?;
                        match value.to_opt_bool() {
                            Some(_value) => writeln!(f, "relevant")?,
                            None => writeln!(f, "irrelevant")?,
                        }
                    }
                },
                StackValue::CloseParen => {
                    column -= 1;
                    tab_to_column(&mut f, column)?;
                    writeln!(f, ")")?;
                }
            }
        }

        writeln!(f, ")")?;
        Ok(())
    }
}

pub struct UnsatValidator<'a> {
    problem: &'a Problem,
    proof: &'a Proof,
    stack: Vec<(usize, Assignment<RBitvector>)>,
}

impl<'a> UnsatValidator<'a> {
    pub fn new(problem: &'a Problem, proof: &'a Proof) -> Self {
        assert!(!proof.nodes.is_empty());

        let stack = vec![(0, problem.unknown_assignment())];

        Self {
            problem,
            proof,
            stack,
        }
    }

    // Validate the proof by a depth-first-search on the nodes.
    //
    // Panics if the proof is invalid.
    pub fn validate(mut self) {
        let mut evaluator = Evaluator::new(self.problem);

        // It suffices to validate that the claimed value of evaluation
        // of leaf nodes reachable from the root is zero and it matches
        // the actual evaluation value.
        //
        // To ensure finite validation time, reject any proofs that may be
        // circular by ensuring that the children nodes have a greater index
        // than their parent.

        while let Some((node_index, assignment)) = self.stack.pop() {
            let node = &self.proof.nodes[node_index];
            match node {
                ProofNode::Decision(decision_node) => {
                    // ensure the children are located after the current node
                    // so the validation is guaranteed to end in finite time
                    assert!(decision_node.child_zero > node_index);
                    assert!(decision_node.child_one > node_index);

                    // apply both phases of the decision to the assignment, which must be yet-undecided there
                    let mut child_zero_assignment = assignment.clone();
                    child_zero_assignment
                        .apply_bool_decision_to_undecided(decision_node.decision, false);

                    let mut child_one_assignment = assignment;
                    child_one_assignment
                        .apply_bool_decision_to_undecided(decision_node.decision, true);

                    // ensure both children with the assignments are validated
                    self.stack
                        .push((decision_node.child_zero, child_zero_assignment));
                    self.stack
                        .push((decision_node.child_one, child_one_assignment));
                }
                ProofNode::Value(value) => {
                    // the value must be zero so this is unsat
                    assert_eq!(*value, ThreeValued::False);
                    // validate that the assignment truly evaluates to zero single-bit bitvector
                    let eval_result = evaluator.evaluate(&assignment);
                    assert_eq!(
                        eval_result.concrete_value(),
                        Some(ConcreteBitvector::from_bool(false))
                    );
                }
            }
        }
    }
}
