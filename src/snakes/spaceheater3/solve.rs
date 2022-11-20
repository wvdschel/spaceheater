use std::{cmp, fmt::Display, time::Instant};

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
    let enemy_count = game.others.len();
    let turn = game.turn;
    let mut root = MaximizingNode::new(game);

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
    let mut last_node_count = 0;
    for current_depth in base_depth..max_depth {
        println!(
            "turn {}: {}ms: starting depth {}",
            turn,
            start.elapsed().as_millis(),
            current_depth,
        );
        let (res, node_count) = root.solve(deadline, current_depth, score_fn, None, None);
        match &res {
            Some((dir, score)) => {
                best_score = res.clone();
                println!(
                    "turn {}: {}ms: completed depth {}, tree has {} nodes: {} {}",
                    turn,
                    start.elapsed().as_millis(),
                    current_depth,
                    node_count,
                    dir,
                    score,
                );
                log!("complete tree for depth {}:\n{}", current_depth, root);
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
        if node_count == last_node_count {
            println!(
                "turn {}: {}ms: tree completed at depth {}",
                turn,
                start.elapsed().as_millis(),
                current_depth - 1,
            );
            break;
        }
        last_node_count = node_count;
    }

    let statm = procinfo::pid::statm_self().unwrap();
    println!(
        "turn {}: {}ms / {} MB: returning {}",
        turn,
        start.elapsed().as_millis(),
        statm.size * 4096 / 1024 / 1024,
        best_score
            .clone()
            .map(|v| v.0.to_string())
            .unwrap_or("None".to_string())
    );

    best_score
}
