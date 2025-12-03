use std::fmt::Debug;

use itertools::Itertools;

use crate::problem::Assignment;

use super::Learned;

#[derive(Clone)]
pub struct RTreeLearned {
    root: Node,
}

const MAXIMUM_ENTRIES: usize = 4;
const MINIMUM_ENTRIES: usize = MAXIMUM_ENTRIES / 2;

impl Learned for RTreeLearned {
    fn new() -> Self {
        Self {
            root: Node::Leaf(Leaf {
                entries: Vec::new(),
            }),
        }
    }

    fn contains(&self, assignment: &Assignment) -> bool {
        self.root.contains(assignment)
    }

    fn add(&mut self, assignment: Assignment) {
        //eprintln!("Inserting {:?}", assignment);
        match self.root.insert(assignment) {
            NodeUpward::Inserted(_assignment) => {
                // do nothing
            }
            NodeUpward::Split(new_node) => {
                // we have to split the root
                // TODO: avoid cloning
                let old_root = self.root.clone();
                let old_root_bound = self.root.compute_bound();
                let new_node_bound = new_node.compute_bound();
                self.root = Node::NonLeaf(NonLeaf {
                    entries: vec![(old_root_bound, old_root), (new_node_bound, new_node)],
                });
            }
        }
    }

    fn write_dot<W: std::io::Write>(&self, f: &mut W) -> std::io::Result<()> {
        writeln!(f, "digraph {{")?;
        writeln!(f, "rankdir=\"LR\"")?;
        writeln!(f, "0 [label=\"root\"]")?;
        self.root.write_dot(f, &mut 0)?;
        writeln!(f, "}}")
    }
}

impl Debug for RTreeLearned {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.root.fmt(f)
    }
}

#[derive(Clone)]
enum Node {
    NonLeaf(NonLeaf),
    Leaf(Leaf),
}

impl Node {
    fn contains(&self, assignment: &Assignment) -> bool {
        match self {
            Node::NonLeaf(non_leaf) => non_leaf.contains(assignment),
            Node::Leaf(leaf) => leaf.contains(assignment),
        }
    }

    fn insert(&mut self, assignment: Assignment) -> NodeUpward {
        match self {
            Node::NonLeaf(non_leaf) => non_leaf.insert(assignment),
            Node::Leaf(leaf) => leaf.insert(assignment),
        }
    }

    fn compute_bound(&self) -> Assignment {
        match self {
            Node::NonLeaf(non_leaf) => non_leaf.compute_bound(),
            Node::Leaf(leaf) => leaf.compute_bound(),
        }
    }

    fn compute_enlargement(&self, assignment: &Assignment) -> i64 {
        match self {
            Node::NonLeaf(non_leaf) => non_leaf.compute_enlargement(assignment),
            Node::Leaf(leaf) => leaf.compute_enlargement(assignment),
        }
    }

    fn write_dot<W: std::io::Write>(&self, f: &mut W, unique: &mut u64) -> std::io::Result<()> {
        let our_unique = *unique;
        *unique += 1;
        match self {
            Node::NonLeaf(non_leaf) => {
                for (entry_bound, entry_node) in &non_leaf.entries {
                    let bound_string = format!("{:?}", entry_bound).replace("\"", "\\\"");
                    writeln!(f, "{} [label=\"{}\"]", unique, bound_string)?;
                    writeln!(f, "{} -> {}", our_unique, unique)?;
                    entry_node.write_dot(f, unique)?;
                }
            }
            Node::Leaf(leaf) => {
                for entry in &leaf.entries {
                    let bound_string = format!("{:?}", entry).replace("\"", "\\\"");
                    writeln!(f, "{} [label=\"{}\"]", unique, bound_string)?;
                    writeln!(f, "{} -> {}", our_unique, unique)?;
                    *unique += 1;
                }
            }
        }
        Ok(())
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonLeaf(arg0) => arg0.fmt(f),
            Self::Leaf(arg0) => arg0.fmt(f),
        }
    }
}

#[derive(Clone)]
struct NonLeaf {
    entries: Vec<(Assignment, Node)>,
}

impl NonLeaf {
    fn contains(&self, assignment: &Assignment) -> bool {
        for (bound, node) in &self.entries {
            // the point: filter out by bounds

            if !bound.contains(assignment) {
                continue;
            }

            if node.contains(assignment) {
                return true;
            }
        }
        false
    }

    fn compute_enlargement(&self, assignment: &Assignment) -> i64 {
        let mut min_enlargement = i64::MAX;

        for (_, entry_node) in self.entries.iter() {
            min_enlargement = min_enlargement.min(entry_node.compute_enlargement(assignment));
        }

        min_enlargement
    }

    fn insert(&mut self, assignment: Assignment) -> NodeUpward {
        // non-leaf node
        let mut chosen = None;

        for (entry_index, (entry_bound, _entry_node)) in self.entries.iter().enumerate() {
            let entry_volume = volume(entry_bound);

            // compute enlargement of leafs
            let enlargement = self.compute_enlargement(&assignment);

            if let Some((_, chosen_volume, chosen_enlargement)) = chosen {
                // prefer smaller enlargement, then same enlargement of smaller volume
                if enlargement < chosen_enlargement
                    || (enlargement == chosen_enlargement && entry_volume < chosen_volume)
                {
                    chosen = Some((entry_index, entry_volume, enlargement));
                }
            } else {
                chosen = Some((entry_index, entry_volume, enlargement));
            }
        }

        let (chosen_index, _, _) = chosen.expect("Some child should be chosen");
        let num_entries = self.entries.len();
        let (chosen_bound, chosen_node) = &mut self.entries[chosen_index];

        // descend

        match chosen_node.insert(assignment) {
            NodeUpward::Inserted(assignment) => {
                // adjust entry assignment
                // TODO: avoid cloning
                *chosen_bound = chosen_bound.clone().join(&assignment);

                // we can just keep returning the assignment to enlarge the ancestors as well
                NodeUpward::Inserted(assignment)
            }
            NodeUpward::Split(new_node) => {
                *chosen_bound = chosen_node.compute_bound();

                let chosen_clone = chosen_bound.clone();

                // propagate node split upward
                // TODO: deduplicate logic

                if num_entries < MAXIMUM_ENTRIES {
                    // insert child
                    let new_node_bound = new_node.compute_bound();
                    let together_bound = chosen_clone.join(&new_node_bound);

                    self.entries.push((new_node_bound, new_node));

                    NodeUpward::Inserted(together_bound)
                } else {
                    // split
                    let new_entry = (new_node.compute_bound(), new_node);
                    let new_entries = split_entries(&mut self.entries, new_entry, |entry| &entry.0);

                    NodeUpward::Split(Node::NonLeaf(NonLeaf {
                        entries: new_entries,
                    }))
                }
            }
        }
    }

    fn compute_bound(&self) -> Assignment {
        compute_assignments_bound(self.entries.iter().map(|(assignment, _)| assignment))
    }
}

impl Debug for NonLeaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut franz = f.debug_map();

        for (bound, node) in &self.entries {
            franz.entry(bound, node);
        }

        franz.finish()
    }
}

#[derive(Clone)]
struct Leaf {
    entries: Vec<Assignment>,
}

enum NodeUpward {
    Inserted(Assignment),
    Split(Node),
}

impl Leaf {
    fn contains(&self, assignment: &Assignment) -> bool {
        // note the reversed logic from non-leaves
        // entries in leaves are an underapproximation
        // while bounds in non-leaves are an overapproximation
        for entry in &self.entries {
            if entry.contains(assignment) {
                return true;
            }
        }
        false
    }

    fn insert(&mut self, assignment: Assignment) -> NodeUpward {
        if self.entries.len() < MAXIMUM_ENTRIES {
            // insert child
            self.entries.push(assignment.clone());
            NodeUpward::Inserted(assignment)
        } else {
            // split
            // TODO: implement reasonable splits
            let new_entries = split_entries(&mut self.entries, assignment, |a| a);
            NodeUpward::Split(Node::Leaf(Leaf {
                entries: new_entries,
            }))
        }
    }

    fn compute_bound(&self) -> Assignment {
        compute_assignments_bound(&self.entries)
    }

    fn compute_enlargement(&self, assignment: &Assignment) -> i64 {
        let mut least_differences = u64::MAX;
        for entry in &self.entries {
            let differences = num_differences(entry, assignment);
            least_differences = least_differences.min(differences);
        }

        least_differences as i64
    }
}

impl Debug for Leaf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.entries).finish()
    }
}

fn compute_assignments_bound<'a>(
    into_iter: impl IntoIterator<Item = &'a Assignment>,
) -> Assignment {
    into_iter
        .into_iter()
        .fold(None, |acc: Option<Assignment>, elem| {
            if let Some(acc) = acc {
                Some(acc.join(elem))
            } else {
                Some(elem.clone())
            }
        })
        .expect("Assignments should have at least one element")
}

fn split_entries<T: Debug, F: Fn(&T) -> &Assignment>(
    our_entries: &mut Vec<T>,
    new_entry: T,
    bound_fn: F,
) -> Vec<T> {
    // Guttman quadratic for now
    // pick seeds
    our_entries.push(new_entry);

    let mut chosen = None;

    let mut first_iter = our_entries.iter().enumerate();
    while let Some((first_index, first)) = first_iter.next() {
        let first_bound = bound_fn(first);
        for (second_index, second) in first_iter.clone() {
            let second_bound = bound_fn(second);
            // calculate inefficiency
            let inefficiency = num_differences(first_bound, second_bound);

            // choose the most inefficient pair
            let replace_chosen = if let Some((chosen_inefficiency, _, _)) = chosen {
                inefficiency > chosen_inefficiency
            } else {
                true
            };

            if replace_chosen {
                chosen = Some((inefficiency, first_index, second_index));
            }
        }
    }

    let (_inefficiency, first_index, second_index) =
        chosen.expect("Most inefficient combination should be chosen");

    // remove second index first to avoid too much shifting and other index renumbering

    let second_seed = our_entries.remove(second_index);
    let mut second_bound = (bound_fn)(&second_seed).clone();
    let mut second_group = vec![second_seed];

    let first_seed = our_entries.remove(first_index);
    let mut first_bound = (bound_fn)(&first_seed).clone();
    let mut first_group = vec![first_seed];

    while !our_entries.is_empty() {
        if our_entries.len() + first_group.len() <= MINIMUM_ENTRIES {
            for entry in our_entries.drain(..) {
                let bound = (bound_fn)(&entry);
                first_bound = bound.clone().join(&first_bound);
                first_group.push(entry);
            }

            break;
        }

        if our_entries.len() + second_group.len() <= MINIMUM_ENTRIES {
            for entry in our_entries.drain(..) {
                let bound = (bound_fn)(&entry);
                second_bound = bound.clone().join(&second_bound);
                second_group.push(entry);
            }

            break;
        }

        let mut chosen = None;

        for (index, entry) in our_entries.iter().enumerate() {
            let bound = (bound_fn)(entry);

            let first_enlargement = num_differences(bound, &first_bound);
            let second_enlargement = num_differences(bound, &second_bound);

            let (enlargement, insert_to_first) = if first_enlargement <= second_enlargement {
                (first_enlargement, true)
            } else {
                (second_enlargement, false)
            };

            let join = if insert_to_first {
                bound.clone().join(&first_bound)
            } else {
                bound.clone().join(&second_bound)
            };

            let replace_chosen = if let Some((chosen_enlargement, _, _, _)) = chosen {
                enlargement < chosen_enlargement
            } else {
                true
            };

            if replace_chosen {
                chosen = Some((enlargement, index, insert_to_first, join))
            }
        }

        let (_, chosen_index, insert_to_first, join) = chosen.expect("Some entry should be chosen");

        let entry = our_entries.remove(chosen_index);

        if insert_to_first {
            first_bound = join;
            first_group.push(entry);
        } else {
            second_bound = join;
            second_group.push(entry);
        }
    }

    assert!(first_group.len() >= MINIMUM_ENTRIES);
    assert!(second_group.len() >= MINIMUM_ENTRIES);

    // assign first group to ours and return second group

    *our_entries = first_group;

    second_group
}

pub fn volume(assignment: &Assignment) -> u64 {
    let mut count = 0;

    for our_value in assignment.values().iter() {
        count += our_value.get_unknown_bits().to_u64().count_ones() as u64;
    }

    count
}

pub fn num_differences(lhs: &Assignment, rhs: &Assignment) -> u64 {
    let mut count = 0;

    for (our_value, rhs_value) in lhs.values().iter().zip_eq(rhs.values().iter()) {
        let our_zeros = our_value.get_possibly_zero_flags().to_u64();
        let our_ones = our_value.get_possibly_one_flags().to_u64();

        let rhs_zeros = rhs_value.get_possibly_zero_flags().to_u64();
        let rhs_ones = rhs_value.get_possibly_one_flags().to_u64();

        let zero_diff = our_zeros ^ rhs_zeros;
        let one_diff = our_ones ^ rhs_ones;
        let some_diff = zero_diff | one_diff;

        count += some_diff.count_ones() as u64;
    }

    count
}
