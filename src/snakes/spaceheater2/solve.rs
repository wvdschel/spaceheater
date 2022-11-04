use std::{fmt::Display, sync::Arc, time::Instant};

use crate::logic::Game;

use super::scores::Scoretree;

pub fn solve<Fscore, Fmin, Fmax, S1, S2, S3>(
    game: &Game,
    expensive_score_fn: &Fscore,
    cheap_min_score_fn: &Fmin,
    cheap_max_score_fn: &Fmax,
    scores: Arc<Scoretree<S1>>,
    deadline: Instant,
    max_depth: usize,
) 
where
    Fscore: Fn(&Game) -> S1,
    Fmin: Fn(&Game) -> S2,
    Fmax: Fn(&Game) -> S3,
    S1: Ord + Display + Clone,
    S2: Ord + PartialEq<S1>,
    S3: Ord + PartialEq<S1>,
{
}
