#[derive(PartialEq, Eq)]
pub struct Inverse<S: Ord> {
    s: S,
}

impl<S: Ord> Ord for Inverse<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.s.cmp(&other.s) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

impl<S: Ord> PartialOrd<Self> for Inverse<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other));
    }
}

pub fn invert<S: Ord>(s: S) -> Inverse<S> {
    Inverse { s }
}
