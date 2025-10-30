pub mod forward;

pub trait Join {
    fn join(self, other: &Self) -> Self;
}
