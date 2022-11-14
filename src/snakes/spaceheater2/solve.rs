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
pub fn solve<Fscore, S1>(
    game: &Game,
    path_so_far: Vec<Direction>,
    score_fn: &Fscore,
    scores: &Scoretree<S1>,
    deadline: Instant,
    max_depth: usize,
    alpha: Option<S1>,
    beta: Option<S1>,
) -> (Vec<Direction>, S1)
where
    Fscore: Fn(&Game) -> S1,
    S1: Ord + Display + Clone + Send + 'static,
{
    // return if dead, return if max depth, return if timeline exceeded
    if game.you.health <= 0 || path_so_far.len() == max_depth || deadline < Instant::now() {
        return (vec![], score_fn(game));
    }

    let mut alpha = alpha;
    let mut beta = beta;

    let mut max_score: Option<S1> = None;
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

        let mut min_score: Option<S1> = None;
        let mut best_min_move = vec![];
        for enemy_moves in all_sensible_enemy_moves(game) {
            let mut successor = game.clone();
            successor.execute_moves(my_dir, &enemy_moves);
            let (next_move, succ_score) = solve(
                &successor,
                path.clone(),
                score_fn,
                scores,
                deadline,
                max_depth,
                alpha.clone(),
                beta.clone(),
            );
            log!(
                "Evaluated {} + {:?} = {}:\n{}\n",
                my_dir,
                enemy_moves,
                succ_score,
                successor
            );

            let min = match &max_score {
                Some(s) => s,
                None => &succ_score,
            };
            if min >= &succ_score {
                best_min_move = next_move;
                min_score = Some(succ_score.clone());

                let beta_score = match &beta {
                    Some(s) => s,
                    None => &succ_score,
                };
                if beta_score >= &succ_score {
                    beta = Some(succ_score);
                }
            }

            if deadline < Instant::now() {
                break;
            }

            if let Some(max_score) = &alpha {
                if let Some(min_score) = &beta {
                    if max_score >= min_score {
                        break;
                    }
                }
            }
        }

        let succ_score = min_score.unwrap();
        let max = match &max_score {
            Some(s) => s,
            None => &succ_score,
        };
        if max <= &succ_score {
            best_move = vec![my_dir];
            best_move.append(&mut best_min_move);
            max_score = Some(succ_score.clone());

            let alpha_score = match &alpha {
                Some(s) => s,
                None => &succ_score,
            };
            if alpha_score <= &succ_score {
                alpha = Some(succ_score);
            }
        }

        if let Some(max_score) = &alpha {
            if let Some(min_score) = &beta {
                if max_score >= min_score {
                    break;
                }
            }
        }
    }

    // TODO also return or store all leaf nodes with their score (for sorting purposes) to implement incremental deepening
    (best_move, max_score.unwrap())
}
