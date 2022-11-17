use std::{cmp, fmt::Display, time::Instant};

use crate::{
    log,
    logic::{Direction, Game},
    protocol::ALL_DIRECTIONS,
    util::invert::invert,
};

use super::util::{all_sensible_enemy_moves, certain_death};

pub struct MaximizingNode<S: Ord + Display + Clone + Send + 'static> {
    game: Game,
    score: Option<(Direction, S)>,
    children: Vec<MinimizingNode<S>>,
}

impl<S: Ord + Display + Clone + Send + 'static> MaximizingNode<S> {
    fn solve_fork<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> Option<(Direction, S)>
    where
        FScore: Fn(&Game) -> S,
    {
        // TODO
        self.solve(deadline, max_depth, score_fn, alpha, beta)
    }

    fn update_children(&mut self) {
        if self.children.len() == 0 {
            self.children = ALL_DIRECTIONS
                .iter()
                .filter(|&my_dir| {
                    let mut my_pos = self.game.you.head.neighbour(*my_dir);
                    self.game.warp(&mut my_pos);
                    !certain_death(&self.game, &self.game.you, &my_pos)
                })
                .map(|my_dir| MinimizingNode {
                    my_move: *my_dir,
                    score: None,
                    children: vec![],
                })
                .collect();
        } else {
            self.children.sort_by(|c1, c2| c1.cmp_scores(c2))
        }
    }

    fn check_bounds<FScore>(&mut self, max_depth: usize, score_fn: &FScore) -> bool
    where
        FScore: Fn(&Game) -> S,
    {
        if self.game.you.dead() {
            if self.score == None {
                self.score = Some((Direction::Up, score_fn(&self.game)));
            }
            return true;
        }
        if max_depth == 0 {
            let score = score_fn(&self.game);
            self.score = Some((Direction::Up, score));
            return true;
        }

        false
    }

    fn solve<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> Option<(Direction, S)>
    where
        FScore: Fn(&Game) -> S,
    {
        if Instant::now() > *deadline {
            return None;
        }

        if self.check_bounds(max_depth, score_fn) {
            return self.score.clone();
        }

        self.update_children();

        if self.children.len() == 0 {
            // All paths are certain death, just score this board and return
            let score = score_fn(&self.game);
            self.score = Some((Direction::Up, score));
            return self.score.clone();
        }

        let mut best_dir = Direction::Up;
        let mut max_score = None;
        let mut alpha = alpha;
        for min_node in &mut self.children {
            let next_score = min_node.solve(
                &mut self.game,
                deadline,
                max_depth,
                score_fn,
                alpha.clone(),
                beta.clone(),
            );

            if next_score == None {
                return None; // Deadline exceeded
            }

            if max_score < next_score {
                best_dir = min_node.my_move;
                max_score = next_score.clone()
            }
            alpha = cmp::max(alpha, next_score);
            if beta != None {
                if alpha > beta {
                    break;
                }
            }
        }

        self.score = max_score.map(|s| (best_dir, s));
        self.score.clone()
    }

    fn cmp_scores(&self, other: &Self) -> cmp::Ordering {
        if self.score == other.score {
            return cmp::Ordering::Equal;
        }
        match &self.score {
            Some((_, self_score)) => match &other.score {
                Some((_, other_score)) => self_score.cmp(other_score),
                None => cmp::Ordering::Less,
            },
            None => cmp::Ordering::Greater,
        }
    }

    fn format_tree(&self, depth: usize) -> String {
        let mut strings = Vec::<String>::new();
        strings.push(format!(
            "{} MAX DEPTH {} ({} children):",
            "#".repeat(depth * 2 + 1),
            depth,
            self.children.len()
        ));
        match &self.score {
            Some((dir, score)) => {
                strings.push(format!("best move is {} with score {}", dir, score))
            }
            None => {}
        };
        strings.push(format!("{}", self.game));

        let mut children: Vec<&MinimizingNode<S>> = self.children.iter().collect();
        children.sort_by(|c1, c2| c1.cmp_scores(c2));
        for c in children {
            strings.push(c.format_tree(depth));
        }

        strings.join("\n")
    }
}

impl<S: Ord + Display + Clone + Send + 'static> std::fmt::Display for MaximizingNode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_tree(0).as_str())
    }
}

pub struct MinimizingNode<S: Ord + Display + Clone + Send + 'static> {
    my_move: Direction,
    score: Option<S>,
    children: Vec<MaximizingNode<S>>,
}

impl<S: Ord + Display + Clone + Send + 'static> MinimizingNode<S> {
    fn solve<FScore>(
        &mut self,
        game: &Game,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> Option<S>
    where
        FScore: Fn(&Game) -> S,
    {
        if self.children.len() == 0 {
            self.children = all_sensible_enemy_moves(game)
                .iter()
                .map(|enemy_moves| {
                    let mut game = game.clone();
                    game.execute_moves(self.my_move, enemy_moves);
                    MaximizingNode {
                        game,
                        score: None,
                        children: vec![],
                    }
                })
                .collect();
        } else {
            self.sort_children()
        }

        let mut min_score = None;
        let mut beta = beta;
        for max_node in &mut self.children {
            let next_score = max_node
                .solve(
                    deadline,
                    max_depth - 1,
                    score_fn,
                    alpha.clone(),
                    beta.clone(),
                )
                .map(|r| r.1);

            if next_score == None {
                return None; // Deadline exceeded
            }

            if min_score != None {
                min_score = cmp::min(min_score, next_score.clone());
            } else {
                min_score = next_score.clone();
            }
            if beta != None {
                beta = cmp::min(beta, next_score);
            } else {
                beta = next_score;
            }
            if beta != None {
                if alpha > beta {
                    break;
                }
            }
        }

        self.score = min_score.clone();
        min_score
    }

    fn sort_children(&mut self) {
        self.children.sort_by(|c1, c2| c1.cmp_scores(c2))
    }

    fn cmp_scores(&self, other: &Self) -> cmp::Ordering {
        if self.score == other.score {
            return cmp::Ordering::Equal;
        }
        match &self.score {
            Some(self_score) => match &other.score {
                Some(other_score) => invert(self_score).cmp(&invert(other_score)),
                None => cmp::Ordering::Less,
            },
            None => cmp::Ordering::Greater,
        }
    }

    fn format_tree(&self, depth: usize) -> String {
        let mut strings = Vec::<String>::new();
        strings.push(format!(
            "{} MIN DEPTH {} ({} children):",
            "#".repeat(depth * 2 + 2),
            depth,
            self.children.len()
        ));
        match &self.score {
            Some(score) => strings.push(format!("best score is {}", score)),
            None => {}
        };

        let mut children: Vec<&MaximizingNode<S>> = self.children.iter().collect();
        children.sort_by(|c1, c2| c1.cmp_scores(c2));
        for c in children {
            strings.push(c.format_tree(depth + 1));
        }

        strings.join(format!("\n").as_str())
    }
}

pub fn solve<Fscore, S>(
    game: Game,
    deadline: &Instant,
    max_depth: usize,
    score_fn: &Fscore,
) -> Option<(Direction, S)>
where
    Fscore: Fn(&Game) -> S,
    S: Ord + Display + Clone + Send + 'static,
{
    let turn = game.turn;
    let mut root = MaximizingNode {
        game,
        score: None,
        children: vec![],
    };

    let base_depth = 3;
    let start = Instant::now();
    let max_depth = cmp::max(base_depth + 1, max_depth);

    println!(
        "turn {}: start: calculating depths {} through {}",
        turn, base_depth, max_depth
    );

    let mut best_score = None;
    for current_depth in base_depth..max_depth {
        println!(
            "turn {}: {}ms: starting depth {}",
            turn,
            start.elapsed().as_millis(),
            current_depth,
        );
        let res = root.solve_fork(deadline, current_depth, score_fn, None, None);
        match &res {
            Some((dir, score)) => {
                best_score = res.clone();
                println!(
                    "turn {}: {}ms: completed depth {}: {} {}",
                    turn,
                    start.elapsed().as_millis(),
                    current_depth,
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
