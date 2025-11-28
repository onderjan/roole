use std::fmt::Debug;

use crate::{
    domain::{
        bitvector::{abstr::BitvectorDomain, concr::ConcreteBitvector},
        value::ThreeValued,
    },
    problem::{assignment::Assignment, decision::Decision},
};

use super::Problem;

/// A solution of the satisfiability problem.
///
/// It either says the problem is satisfiable, producing
/// an assignment that satisfies the problem (a model),
/// or says that the problem is unsatisfiable, producing
/// an unsatisfiability proof.
#[derive(Debug)]
pub enum Solution {
    Satisfiable(Assignment),
    Unsatisfiable(Proof),
}

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

impl Solution {
    /// Validates (proof-checks) that the solution to a problem is correct.
    pub fn validate(&self, problem: &Problem) {
        match self {
            Solution::Satisfiable(claimed_sat_assignment) => {
                // it is claimed the assignment satisfies the problem formula
                // just evaluate the assignment and validate it returns single-bit bitvector 1
                let eval_result = problem.eval(claimed_sat_assignment);
                assert_eq!(
                    eval_result.concrete_value(),
                    Some(ConcreteBitvector::from_bool(true))
                );
            }
            Solution::Unsatisfiable(unsat_proof) => {
                // validate the proof
                UnsatValidator::new(problem, unsat_proof).validate();
            }
        }
    }
}

impl Proof {
    pub fn new(nodes: Vec<ProofNode>) -> Self {
        Self { nodes }
    }
}

struct UnsatValidator<'a> {
    problem: &'a Problem,
    proof: &'a Proof,
    stack: Vec<(usize, Assignment)>,
}

impl<'a> UnsatValidator<'a> {
    fn new(problem: &'a Problem, proof: &'a Proof) -> Self {
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
    fn validate(mut self) {
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
                    let eval_result = self.problem.eval(&assignment);
                    assert_eq!(
                        eval_result.concrete_value(),
                        Some(ConcreteBitvector::from_bool(false))
                    );
                }
            }
        }
    }
}
