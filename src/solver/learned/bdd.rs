use std::collections::HashMap;

use num::{BigUint, Zero};

use crate::{
    assignment::Assignment,
    domain::bitvector::{BitvectorBound, abstr::BitvectorDomain},
    solver::Learned,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct LearnedNode {
    var: u64,
    low: isize,
    high: isize,
}

pub struct BddLearned {
    bdd_list: Vec<LearnedNode>,
    bdd_unique: HashMap<LearnedNode, usize>,
    bdd_index: Option<isize>,
}

impl Learned for BddLearned {
    fn new() -> Self {
        Self {
            bdd_list: vec![],
            bdd_unique: HashMap::new(),
            bdd_index: None,
        }
    }

    fn add(&mut self, assignment: &Assignment) {
        let assignment_bdd_index = self.bdd_state(assignment);

        if let Some(bdd_index) = self.bdd_index {
            self.bdd_index = Some(self.bdd_union(bdd_index, assignment_bdd_index));
        } else {
            self.bdd_index = Some(assignment_bdd_index);
        }
    }

    fn contains(&self, _assignment: &Assignment) -> bool {
        todo!("BDD contains")
    }

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> {
        let Some(bdd_index) = self.bdd_index else {
            return Ok(());
        };

        let mut open = vec![bdd_index];
        let mut closed = Vec::new();

        writeln!(f, "digraph G {{")?;

        while let Some(index) = open.pop() {
            if closed.contains(&index) {
                continue;
            }

            if index == NODE_ZERO_INDEX {
                writeln!(f, "{} [label=\"f\"]", index)?;
                continue;
            } else if index == NODE_ONE_INDEX {
                writeln!(f, "{} [label=\"t\"]", index)?;
                continue;
            }

            let bdd = &self.bdd_list[index as usize];

            writeln!(f, "{} [label=\"{}\"]", index, bdd.var)?;
            writeln!(f, "{} -> {}", index, bdd.high)?;
            writeln!(f, "{} -> {} [style=\"dashed\"]", index, bdd.low)?;

            open.push(bdd.low);
            open.push(bdd.high);
            closed.push(index);
        }

        writeln!(f, "}}\n\n\n")?;

        Ok(())
    }
}

impl BddLearned {
    fn bdd_state(&mut self, assignment: &Assignment) -> isize {
        let (zeros, ones, total_width) = assignment_nums(assignment);

        let mut bdd_index = NODE_ONE_INDEX;

        for i in (0..total_width).rev() {
            let zero = zeros.bit(i);
            let one = ones.bit(i);

            let new_node = match (zero, one) {
                (true, true) => {
                    // both possible, do nothing
                    None
                }
                (true, false) => Some(LearnedNode {
                    var: i,
                    low: bdd_index,
                    high: NODE_ZERO_INDEX,
                }),
                (false, true) => Some(LearnedNode {
                    var: i,
                    low: NODE_ZERO_INDEX,
                    high: bdd_index,
                }),
                (false, false) => panic!("Three-valued bit must be either zero or one"),
            };

            if let Some(new_node) = new_node {
                assert_ne!(new_node.low, new_node.high);

                if let Some(unique_index) = self.bdd_unique.get(&new_node) {
                    bdd_index = *unique_index as isize;
                } else {
                    self.bdd_list.push(new_node);
                    self.bdd_unique.insert(new_node, self.bdd_list.len() - 1);
                    bdd_index = (self.bdd_list.len() - 1) as isize;
                }
            }
        }

        bdd_index
    }

    fn bdd_union(&mut self, left: isize, right: isize) -> isize {
        if left == right {
            return left;
        }

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

        assert_ne!(left, right);

        let left_node = self.bdd_list[left as usize];
        let right_node = self.bdd_list[right as usize];

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

        if node.low == node.high {
            return node.low;
        }

        if let Some(unique_index) = self.bdd_unique.get(&node) {
            *unique_index as isize
        } else {
            assert_ne!(node.low, node.high);

            self.bdd_list.push(node);
            self.bdd_unique.insert(node, self.bdd_list.len() - 1);
            (self.bdd_list.len() - 1) as isize
        }
    }
}

const NODE_ZERO_INDEX: isize = -2isize;
const NODE_ONE_INDEX: isize = -1isize;

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
