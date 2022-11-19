use std::{cmp, fmt::Display, time::Instant};

use bumpalo::Bump;

use crate::{
    log,
    logic::{Direction, Game},
    snakes::spaceheater3::max::MaximizingNode,
};

pub fn solve<FScore, S>(
    game: Game,
    deadline: &Instant,
    max_depth: usize,
    score_fn: &FScore,
) -> Option<(Direction, S)>
where
    FScore: Fn(&Game) -> S,
    S: Ord + Display + Clone + Send + 'static,
{
    let bump = Bump::new();
    let enemy_count = game.others.len();
    let turn = game.turn;
    let root = MaximizingNode::new(game, &bump);

    let base_depth = match enemy_count {
        0 => 3,
        1 => 3,
        2 => 2,
        3 => 2,
        4 => 2,
        _ => 1,
    };
    let start = Instant::now();
    let max_depth = cmp::max(base_depth + 1, max_depth);

    println!(
        "turn {}: start: calculating depths {} through {}",
        turn, base_depth, max_depth
    );

    let mut best_score = None;
    for current_depth in base_depth..max_depth {
        let mut score_count: usize = 0;
        let mut score_fn = |game: &Game| {
            score_count += 1;
            score_fn(game)
        };
        println!(
            "turn {}: {}ms: starting depth {}",
            turn,
            start.elapsed().as_millis(),
            current_depth,
        );
        let res = root.solve_fork(deadline, current_depth, &mut score_fn, None, None);
        match &res {
            Some((dir, score)) => {
                best_score = res.clone();
                println!(
                    "turn {}: {}ms: completed depth {}, evaluated {} games: {} {}",
                    turn,
                    start.elapsed().as_millis(),
                    current_depth,
                    score_count,
                    dir,
                    score,
                );
                //log!("complete tree for depth {}:\n{}", current_depth, root);
                log!(
                    "number of nodes in the tree for depth {}: {}",
                    current_depth,
                    root.len()
                );
            }
            None => {
                println!(
                    "turn {}: {}ms: aborted depth {}",
                    turn,
                    start.elapsed().as_millis(),
                    current_depth
                );
                break;
            }
        }
    }

    println!(
        "turn {}: {}ms: returning {}",
        turn,
        start.elapsed().as_millis(),
        best_score
            .clone()
            .map(|v| v.0.to_string())
            .unwrap_or("None".to_string())
    );

    best_score
}
