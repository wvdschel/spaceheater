use std::{fmt::Display, time::Instant};

use crate::{
    log,
    logic::{Direction, Game},
    protocol::ALL_DIRECTIONS,
};

use super::{
    scores::Scoretree,
    util::{all_sensible_enemy_moves, certain_death},
};

// TODO pass min/max bound for alpha-beta?
pub fn solve<Fscore, Fmin, Fmax, S1, S2, S3>(
    game: &Game,
    path_so_far: Vec<Direction>,
    expensive_score_fn: &Fscore,
    cheap_min_score_fn: &Fmin,
    cheap_max_score_fn: &Fmax,
    scores: &Scoretree<S1>,
    deadline: Instant,
    max_depth: usize,
    parent_min_score: Option<S1>,
) -> (Vec<Direction>, S1)
where
    Fscore: Fn(&Game) -> S1,
    Fmin: Fn(&Game) -> S2,
    Fmax: Fn(&Game) -> S3,
    S1: Ord + Display + Clone + Send + 'static,
    S2: Ord + PartialOrd<S1>,
    S3: Ord + PartialOrd<S1>,
{
    // return if dead, return if max depth, return if timeline exceeded
    if game.you.health <= 0 || path_so_far.len() == max_depth || deadline < Instant::now() {
        return (vec![], expensive_score_fn(game));
    }

    let mut max_score = None;
    let mut best_move = vec![];
    for my_dir in ALL_DIRECTIONS {
        log!("Evaluating {}", my_dir);
        let mut my_pos = game.you.head.neighbour(my_dir);
        game.warp(&mut my_pos);
        let mut path = path_so_far.clone();
        path.push(my_dir);

        if certain_death(game, &game.you, &my_pos) {
            continue;
        }

        let mut min_score = None;
        let mut best_min_move = vec![];
        for enemy_moves in all_sensible_enemy_moves(game) {
            let mut successor = game.clone();
            successor.execute_moves(my_dir, &enemy_moves);
            let (next_move, succ_score) = solve(
                &successor,
                path.clone(),
                expensive_score_fn,
                cheap_min_score_fn,
                cheap_max_score_fn,
                scores,
                deadline,
                max_depth,
                min_score.clone(),
            );
            log!(
                "Evaluated {} + {:?} = {}:\n{}\n",
                my_dir,
                enemy_moves,
                succ_score,
                successor
            );

            if let Some(min) = &min_score {
                if min > &succ_score {
                    best_min_move = next_move;
                    min_score = Some(succ_score)
                }
            } else {
                best_min_move = next_move;
                min_score = Some(succ_score);
            }

            // TODO instrument: break loop if deadline exceeded
            if deadline < Instant::now() {
                break;
            }
            // TODO instrument: break if min score is smaller than max score (ab pruning)
            if let Some(max_score) = &max_score {
                if let Some(min_score) = &min_score {
                    if min_score <= max_score {
                        break;
                    }
                }
            }
        }

        let min_score = min_score.unwrap();
        max_score = Some(match max_score {
            Some(max) => {
                if max < min_score {
                    best_move = vec![my_dir];
                    best_move.append(&mut best_min_move);
                    min_score
                } else {
                    max
                }
            }
            None => min_score,
        });
        // TODO instrument: ab pruning
        if let Some(max_score) = &max_score {
            if let Some(min_score) = &parent_min_score {
                if max_score >= min_score {
                    break;
                }
            }
        }
    }

    // TODO also return or store all leaf nodes with their score (for sorting purposes) to implement incremental deepening
    (best_move, max_score.unwrap())
}
