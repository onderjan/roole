use num::{BigUint, Zero};

use crate::{
    check::Assignment,
    domain::bitvector::{BitvectorBound, abstr::BitvectorDomain},
};

pub struct Learned {
    assignments: Vec<Assignment>,

    bdd: Vec<LearnedNode>,
    bdd_index: Option<isize>,
}

#[derive(Debug, Clone, Copy)]
struct LearnedNode {
    var: u64,
    low: isize,
    high: isize,
}

const NODE_ZERO_INDEX: isize = -2isize;
const NODE_ONE_INDEX: isize = -1isize;

impl Learned {
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            bdd: vec![],
            bdd_index: None,
        }
    }

    pub fn print(&self) {
        //println!("BDD: {:#?}", self.bdd);

        for (index, bdd) in self.bdd.iter().enumerate() {
            println!("{} [label=\"{}\"]", index, bdd.var);
            println!("{} -> {} [style=\"dashed\"]", index, bdd.low);
            println!("{} -> {}", index, bdd.high);
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

        let assignment_bdd_index = self.bdd_state(assignment);

        if let Some(bdd_index) = self.bdd_index {
            self.bdd_index = Some(self.bdd_union(bdd_index, assignment_bdd_index));
        } else {
            self.bdd_index = Some(assignment_bdd_index);
        }

        self.assignments.push(assignment.clone());
    }

    fn bdd_state(&mut self, assignment: &Assignment) -> isize {
        let (zeros, ones, total_width) = assignment_nums(assignment);

        let mut bdd_index = NODE_ONE_INDEX;

        for i in (0..total_width).rev() {
            let zero = zeros.bit(i);
            let one = ones.bit(i);

            match (zero, one) {
                (true, true) => {
                    // both possible, do nothing
                }
                (true, false) => {
                    self.bdd.push(LearnedNode {
                        var: i,
                        low: bdd_index,
                        high: NODE_ZERO_INDEX,
                    });

                    bdd_index = (self.bdd.len() - 1) as isize;
                }
                (false, true) => {
                    self.bdd.push(LearnedNode {
                        var: i,
                        low: NODE_ZERO_INDEX,
                        high: bdd_index,
                    });

                    bdd_index = (self.bdd.len() - 1) as isize;
                }
                (false, false) => panic!("Three-valued bit must be either zero or one"),
            }
        }

        bdd_index
    }

    fn bdd_union(&mut self, left: isize, right: isize) -> isize {
        if left == NODE_ZERO_INDEX && right == NODE_ZERO_INDEX {
            return NODE_ZERO_INDEX;
        }

        if left == NODE_ONE_INDEX || right == NODE_ONE_INDEX {
            return NODE_ONE_INDEX;
        }
        if left == NODE_ZERO_INDEX {
            return right;
        }
        if right == NODE_ZERO_INDEX {
            return left;
        }

        assert!(left >= 0);
        assert!(right >= 0);

        let left_node = self.bdd[left as usize];
        let right_node = self.bdd[right as usize];

        let node = if left_node.var < right_node.var {
            LearnedNode {
                var: left_node.var,
                low: self.bdd_union(left_node.low, right),
                high: self.bdd_union(left_node.high, right),
            }
        } else if left_node.var > right_node.var {
            LearnedNode {
                var: right_node.var,
                low: self.bdd_union(left, right_node.low),
                high: self.bdd_union(left, right_node.high),
            }
        } else {
            LearnedNode {
                var: left_node.var,
                low: self.bdd_union(left_node.low, right_node.low),
                high: self.bdd_union(left_node.high, right_node.high),
            }
        };

        self.bdd.push(node);
        (self.bdd.len() - 1) as isize
    }
}

fn assignment_nums(assignment: &Assignment) -> (BigUint, BigUint, u64) {
    let mut zeros = BigUint::zero();
    let mut ones = BigUint::zero();

    let mut total_width = 0;

    for value in assignment.values.iter().rev() {
        let width = value.bound().width();
        total_width += width as u64;

        zeros <<= width;
        ones <<= width;

        zeros += value.get_possibly_zero_flags().to_u64();
        ones += value.get_possibly_one_flags().to_u64();
    }

    (zeros, ones, total_width)
}
