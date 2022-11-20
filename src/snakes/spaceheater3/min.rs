use std::{cmp, fmt::Display, time::Instant};

use crate::{
    logic::{Direction, Game},
    util::invert::invert,
};

use super::{max::MaximizingNode, util::all_sensible_enemy_moves};

pub struct MinimizingNode<S: Ord + Display + Clone + Send + 'static> {
    pub my_move: Direction,
    score: Option<S>,
    children: Vec<MaximizingNode<S>>,
}

impl<'a, S: Ord + Display + Clone + Send + 'static> MinimizingNode<S> {
    pub fn new(my_move: Direction) -> Self {
        Self {
            my_move,
            score: None,
            children: vec![],
        }
    }

    pub fn solve<FScore>(
        &mut self,
        game: &Game,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
    ) -> (Option<S>, usize)
    where
        FScore: Fn(&Game) -> S,
    {
        if self.children.len() == 0 {
            for combo in all_sensible_enemy_moves(game) {
                let mut game = game.clone();
                game.execute_moves(self.my_move, &combo);
                self.children.push(MaximizingNode::new(game));
            }
        } else {
            self.sort_children()
        }

        let mut min_score = None;
        let mut beta = beta;
        let mut total_node_count = 0;
        for max_node in &mut self.children {
            let (next_score, node_count) = max_node.solve(
                deadline,
                max_depth - 1,
                score_fn,
                alpha.clone(),
                beta.clone(),
            );

            let next_score = next_score.map(|r| r.1);

            total_node_count += node_count;

            if next_score == None {
                return (None, total_node_count); // Deadline exceeded
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
        (min_score, total_node_count)
    }

    fn sort_children(&mut self) {
        self.children.sort_by(|c1, c2| c1.cmp_scores(c2))
    }

    pub fn cmp_scores(&self, other: &Self) -> cmp::Ordering {
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

    pub fn format_tree(&self, depth: usize) -> String {
        let mut strings = std::vec::Vec::<String>::new();
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

        let mut children: std::vec::Vec<&MaximizingNode<S>> = self.children.iter().collect();
        children.sort_by(|c1, c2| c1.cmp_scores(c2));
        for c in children {
            strings.push(c.format_tree(depth + 1));
        }

        strings.join(format!("\n").as_str())
    }

    pub fn len(&self) -> usize {
        let mut len = 1;
        for c in &self.children {
            len += c.len()
        }
        len
    }
}
