pub mod forward;

pub trait Join: Clone + PartialEq + Eq {
    fn join(self, other: &Self) -> Self;

    fn apply_join(&mut self, other: &Self) {
        // default implementation clones
        // this is a non-issue for Copy types
        *self = self.clone().join(other);
    }

    fn contains(&self, contained: &Self) -> bool {
        // default implementation clones
        // this is a non-issue for Copy types
        self.clone().join(contained) == *self
    }
}
