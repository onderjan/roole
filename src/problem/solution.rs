use std::fmt::Debug;

use crate::{
    domain::{
        bitvector::{abstr::BitvectorDomain, concr::ConcreteBitvector},
        value::ThreeValued,
    },
    problem::{assignment::Assignment, decision::Decision},
};

use super::Problem;

#[derive(Debug)]
pub enum Solution {
    Satisfiable(Assignment),
    Unsatisfiable(Proof),
}

#[derive(Debug)]
pub struct Proof {
    nodes: Vec<ProofNode>,
}

#[derive(Debug)]
pub enum ProofNode {
    Decision(ProofDecisionNode),
    Value(ThreeValued),
}

#[derive(Debug)]
pub struct ProofDecisionNode {
    pub decision: Decision,
    pub child_zero: usize,
    pub child_one: usize,
}

impl Solution {
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

    fn validate(mut self) {
        while let Some((node_index, assignment)) = self.stack.pop() {
            let node = &self.proof.nodes[node_index];
            match node {
                ProofNode::Decision(decision_node) => {
                    // ensure the children are located after the current node
                    // so the proof completes in finite time
                    assert!(decision_node.child_zero > node_index);
                    assert!(decision_node.child_one > node_index);

                    // apply both phases of the decision to the assignment, which must be yet-undecided there
                    let mut child_zero_assignment = assignment.clone();
                    child_zero_assignment
                        .apply_decision_to_undecided(decision_node.decision, false);

                    let mut child_one_assignment = assignment;
                    child_one_assignment.apply_decision_to_undecided(decision_node.decision, true);

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
