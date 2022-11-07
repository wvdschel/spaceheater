use std::{fmt::Display, time::Instant};

use crate::{
    logic::{Direction, Game},
    protocol::ALL_DIRECTIONS,
};

use super::{scores::Scoretree, util::certain_death};

pub fn solve<Fscore, Fmin, Fmax, S1, S2, S3>(
    game: &Game,
    path_so_far: Vec<Direction>,
    expensive_score_fn: &Fscore,
    cheap_min_score_fn: &Fmin,
    cheap_max_score_fn: &Fmax,
    scores: Scoretree<S1>,
    deadline: Instant,
    max_depth: usize,
) where
    Fscore: Fn(&Game) -> S1,
    Fmin: Fn(&Game) -> S2,
    Fmax: Fn(&Game) -> S3,
    S1: Ord + Display + Clone + Send + 'static,
    S2: Ord + PartialOrd<S1>,
    S3: Ord + PartialOrd<S1>,
{
    if let Some(current_score) = scores.get_score(&path_so_far) {
        if cheap_max_score_fn(game) < current_score {
            // TODO: This game cannot possibly lead to a new top score, don't continue?
        }
    }

    for my_dir in ALL_DIRECTIONS {
        let my_pos = game.you.head.neighbour(my_dir);
        if certain_death(game, &game.you, &my_pos, game.you.health) {
            // TODO: skip this direction, unless all paths are certain death?
        }
    }
}
