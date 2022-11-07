use crate::logic::Game;

#[derive(Ord,PartialOrd,PartialEq,Eq)]
pub struct NoMinPruningScore {}
#[derive(Ord,PartialOrd,PartialEq,Eq)]
pub struct NoMaxPruningScore {}

pub fn no_max_pruning(_: &Game) -> NoMaxPruningScore {
    NoMaxPruningScore{}
}

pub fn no_min_pruning(_: &Game) -> NoMinPruningScore {
    NoMinPruningScore{}
}

#[macro_export]
macro_rules! no_pruning {
    ($score:ty) => {
        impl PartialEq<$score> for crate::logic::scoring::pruning::NoMinPruningScore {
            fn eq(&self, _other: &$score) -> bool {
                return false;
            }
        }

        impl PartialOrd<$score> for crate::logic::scoring::pruning::NoMinPruningScore {
            fn partial_cmp(&self, _other: &$score) -> Option<std::cmp::Ordering> {
                Some(std::cmp::Ordering::Less)
            }
        }

        impl PartialEq<$score> for crate::logic::scoring::pruning::NoMaxPruningScore {
            fn eq(&self, _other: &$score) -> bool {
                return false;
            }
        }

        impl PartialOrd<$score> for crate::logic::scoring::pruning::NoMaxPruningScore {
            fn partial_cmp(&self, _other: &$score) -> Option<std::cmp::Ordering> {
                Some(std::cmp::Ordering::Greater)
            }
        }
    }
}