use std::{cmp, fmt::Display, time::Instant};

use crate::{
    logic::{Direction, Game},
    protocol::ALL_DIRECTIONS,
};

use super::{min::MinimizingNode, util::certain_death};

pub struct MaximizingNode<S: Ord + Display + Clone + Send + 'static> {
    game: Game,
    score: Option<(Direction, S)>,
    children: Vec<MinimizingNode<S>>,
}

impl<S: Ord + Display + Clone + Send + 'static> MaximizingNode<S> {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            score: None,
            children: vec![],
        }
    }

    pub fn solve_fork<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &mut FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> Option<(Direction, S)>
    where
        FScore: FnMut(&Game) -> S,
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
                .map(|my_dir| MinimizingNode::new(*my_dir))
                .collect();
        } else {
            self.children.sort_by(|c1, c2| c1.cmp_scores(c2))
        }
    }

    fn check_bounds<FScore>(&mut self, max_depth: usize, score_fn: &mut FScore) -> bool
    where
        FScore: FnMut(&Game) -> S,
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

    pub fn solve<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &mut FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> Option<(Direction, S)>
    where
        FScore: FnMut(&Game) -> S,
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

    pub fn cmp_scores(&self, other: &Self) -> cmp::Ordering {
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

    pub fn format_tree(&self, depth: usize) -> String {
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

    #[allow(unused)]
    pub fn len(&self) -> usize {
        let mut len = 1;
        for c in &self.children {
            len += c.len()
        }
        len
    }
}

impl<S: Ord + Display + Clone + Send + 'static> std::fmt::Display for MaximizingNode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_tree(0).as_str())
    }
}
