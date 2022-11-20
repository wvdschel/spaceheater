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

impl<'a, S: Ord + Display + Clone + Send + 'static> MaximizingNode<S> {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            score: None,
            children: vec![],
        }
    }

    pub fn solve<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> (Option<(Direction, S)>, usize)
    where
        FScore: Fn(&Game) -> S,
    {
        if Instant::now() > *deadline {
            return (None, 0);
        }

        if self.check_bounds(max_depth, score_fn) {
            return (self.score.clone(), 1);
        }

        self.update_children();

        if self.children.len() == 0 {
            // All paths are certain death, just score this board and return
            let score = score_fn(&self.game);
            self.score = Some((Direction::Up, score));
            return (self.score.clone(), 1);
        }

        let mut best_dir = Direction::Up;
        let mut max_score = None;
        let mut alpha = alpha;
        let mut total_node_count = 0;
        for min_node in &mut self.children {
            let (next_score, node_count) = min_node.solve(
                &mut self.game,
                deadline,
                max_depth,
                score_fn,
                alpha.clone(),
                beta.clone(),
            );
            total_node_count += node_count;

            if next_score == None {
                return (None, total_node_count); // Deadline exceeded
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
        (self.score.clone(), total_node_count)
    }

    fn update_children(&mut self) {
        if self.children.len() == 0 {
            for my_dir in ALL_DIRECTIONS {
                let mut my_pos = self.game.you.head.neighbour(my_dir);
                self.game.warp(&mut my_pos);
                if certain_death(&self.game, &self.game.you, &my_pos) {
                    continue;
                }
                self.children.push(MinimizingNode::new(my_dir));
            }
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
        let mut strings = std::vec::Vec::<String>::new();
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

        let mut children: std::vec::Vec<&MinimizingNode<S>> = self.children.iter().collect();
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

impl<'a, S: Ord + Display + Clone + Send + 'static> std::fmt::Display for MaximizingNode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_tree(0).as_str())
    }
}
