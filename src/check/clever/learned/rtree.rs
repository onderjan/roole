use std::fmt::Debug;

use crate::check::Assignment;

#[derive(Debug, Clone)]
pub struct RTree {
    root: Node,
}

const MAXIMUM_ENTRIES: usize = 8;
const MINIMUM_ENTRIES: usize = MAXIMUM_ENTRIES / 2;
//const MINIMUM_ENTRIES: usize = 4;

impl RTree {
    pub fn new() -> Self {
        Self {
            root: Node::Leaf(Leaf {
                entries: Vec::new(),
            }),
        }
    }

    pub fn contains(&self, assignment: &Assignment) -> bool {
        self.root.contains(assignment)
    }

    pub fn insert(&mut self, assignment: Assignment) {
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

    pub fn print_dot(&self) {
        //println!("{:#?}", self);

        println!("digraph {{");
        println!("rankdir=\"LR\"");
        println!("0 [label=\"root\"]");
        self.root.print_dot(&mut 0);
        println!("}}");
    }
}

#[derive(Debug, Clone)]
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

    fn print_dot(&self, unique: &mut u64) {
        let our_unique = *unique;
        *unique += 1;
        match self {
            Node::NonLeaf(non_leaf) => {
                for (entry_bound, entry_node) in &non_leaf.entries {
                    let bound_string = format!("{:?}", entry_bound).replace("\"", "\\\"");
                    println!("{} [label=\"{}\"]", unique, bound_string);
                    println!("{} -> {}", our_unique, unique);
                    entry_node.print_dot(unique);
                }
            }
            Node::Leaf(leaf) => {
                for entry in &leaf.entries {
                    let bound_string = format!("{:?}", entry).replace("\"", "\\\"");
                    println!("{} [label=\"{}\"]", unique, bound_string);
                    println!("{} -> {}", our_unique, unique);
                    *unique += 1;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
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

    fn insert(&mut self, assignment: Assignment) -> NodeUpward {
        // non-leaf node
        let mut chosen = None;

        for (entry_index, (entry_bound, _entry_node)) in self.entries.iter().enumerate() {
            let entry_volume = entry_bound.volume();
            let join = assignment.clone().join(entry_bound);
            let enlargement = join
                .volume()
                .checked_sub(entry_volume)
                .expect("Join volume should be at least as big as before");

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

                // propagate node split upward
                // TODO: deduplicate logic

                if num_entries < MAXIMUM_ENTRIES {
                    // insert child
                    let new_node_bound = new_node.compute_bound();
                    let together_bound = chosen_bound.clone().join(&new_node_bound);

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

#[derive(Debug, Clone)]
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

    //eprintln!("Splitting: {:?}", our_entries);

    let mut chosen = None;

    let mut first_iter = our_entries.iter().enumerate();
    while let Some((first_index, first)) = first_iter.next() {
        let first_bound = bound_fn(first);
        for (second_index, second) in first_iter.clone() {
            let second_bound = bound_fn(second);
            // calculate inefficiency
            let first_volume = first_bound.volume();
            let second_volume = second_bound.volume();
            let join_volume = first_bound.clone().join(second_bound).volume();

            let inefficiency =
                //(1i64 << join_volume) - (1i64 << first_volume) - (1i64 << second_volume);
                join_volume as i64 - first_volume as i64 - second_volume as i64;

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

    //eprintln!("Inefficiency: {}", inefficiency);

    // remove second index first to avoid too much shifting and other index renumbering

    let second_seed = our_entries.remove(second_index);
    let mut second_bound = (bound_fn)(&second_seed).clone();
    let mut second_group = vec![second_seed];

    let first_seed = our_entries.remove(first_index);
    let mut first_bound = (bound_fn)(&first_seed).clone();
    let mut first_group = vec![first_seed];

    let mut remaining_entries = our_entries.len();

    for entry in our_entries.drain(..) {
        /*eprintln!(
            "F: {:?} S: {:?} ({}, {}, {})",
            first_group,
            second_group,
            first_group.len(),
            second_group.len(),
            MINIMUM_ENTRIES
        );*/
        let (insert_to_first, join) = if remaining_entries + first_group.len() <= MINIMUM_ENTRIES {
            //eprintln!("First");
            let first_join = (bound_fn)(&entry).clone().join(&first_bound);
            (true, first_join)
        } else if remaining_entries + second_group.len() <= MINIMUM_ENTRIES {
            //eprintln!("Second");
            let second_join = (bound_fn)(&entry).clone().join(&second_bound);
            (false, second_join)
        } else {
            //eprintln!("Decide");
            let bound = (bound_fn)(&entry);
            // compute which group will be enlarged the least
            let first_join = bound.clone().join(&first_bound);
            let second_join = bound.clone().join(&second_bound);

            let first_volume = first_bound.volume();
            let second_volume = second_bound.volume();

            let first_enlargment = first_join.volume() - first_volume;
            let second_enlargment = second_join.volume() - second_volume;

            // prefer least enlargement, then smaller volume, then fewer entries

            let insert_to_first = (first_enlargment, first_volume, first_group.len())
                <= (second_enlargment, second_volume, second_group.len());

            if insert_to_first {
                (true, first_join)
            } else {
                (false, second_join)
            }
        };

        if insert_to_first {
            first_bound = join;
            first_group.push(entry);
        } else {
            second_bound = join;
            second_group.push(entry);
        }

        remaining_entries -= 1;
    }

    /*eprintln!("First group: {:?}", first_group);
    eprintln!("Second group: {:?}", second_group);
    eprintln!("Split: {}/{}", first_group.len(), second_group.len());*/

    assert!(first_group.len() >= MINIMUM_ENTRIES);
    assert!(second_group.len() >= MINIMUM_ENTRIES);

    // assign first group to ours and return second group

    *our_entries = first_group;

    second_group
}
