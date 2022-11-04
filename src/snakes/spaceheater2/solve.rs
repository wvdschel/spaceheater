use std::{fmt::Display, sync::Arc, time::Instant};

use crate::logic::Game;

use super::scores::Scoretree;

pub fn solve<F1, F2, S1, S2>(
    game: &Game,
    cheap_scoring_fn: &F1,
    expensive_scoring_fn: &F2,
    scores: Arc<Scoretree<S2>>,
    deadline: Instant,
    max_depth: usize,
) where
    F1: Fn(&Game) -> S1,
    F2: Fn(&Game) -> S2,
    S1: Ord + PartialEq<S2>,
    S2: Ord + Display + Clone,
{
}
