use rayon::prelude::*;

use std::{
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

use crate::logic::{self, Direction, Game};

use super::{alphabeta::AlphaBeta, max::MaximizingNode, util::all_sensible_enemy_moves};

pub struct MinimizingNode {
    pub my_move: Direction,
    pub(super) score: Option<i64>,
    pub(super) children: Vec<MaximizingNode>,
    pub(super) will_die: bool,
}

impl MinimizingNode {
    pub fn new(my_move: Direction) -> Self {
        Self {
            my_move,
            score: None,
            children: vec![],
            will_die: false,
        }
    }

    fn update_children(&mut self, game: &Game) {
        if self.children.len() == 0 {
            for combo in all_sensible_enemy_moves(game) {
                let mut game = game.clone();
                game.execute_moves(self.my_move, &combo);
                self.children.push(MaximizingNode::new(game));
            }
        } else {
            self.children.sort_unstable_by(|c1, c2| c1.cmp_scores(c2));
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

        for c in self.children.iter() {
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

    pub fn cmp_scores(&self, other: &Self) -> std::cmp::Ordering {
        let self_score = self.score.unwrap_or(i64::MIN);
        let other_score = other.score.unwrap_or(i64::MIN);
        other_score.cmp(&self_score)
    }
}

impl MinimizingNode {
    pub fn solve<S>(
        &mut self,
        game: Arc<&Game>,
        deadline: &Instant,
        max_depth: usize,
        scorer: &S,
        alpha_beta: &AlphaBeta<'_>,
        threads: f32,
    ) -> (Option<i64>, usize)
    where
        S: logic::scoring::Scorer + Sync + Clone + 'static,
    {
        let game = *game.as_ref();

        self.update_children(game);
        let (parallel, threads) = if threads > 1f32 {
            (true, threads / self.children.len() as f32)
        } else {
            (false, threads)
        };

        let min_score = AtomicI64::new(i64::MAX);
        let will_die = AtomicBool::new(false);
        let alpha_beta = alpha_beta.new_child();
        let total_node_count = AtomicUsize::new(0);

        let solver = |max_node: &mut MaximizingNode| {
            if alpha_beta.should_be_pruned() {
                return;
            }

            let (next_score, node_count) =
                max_node.solve(deadline, max_depth - 1, scorer, &alpha_beta, threads);

            let next_score = if let Some(s) = next_score {
                s.1
            } else {
                return; // Deadline exceeded
            };

            total_node_count.fetch_add(node_count, Ordering::Relaxed);
            if min_score.fetch_min(next_score, Ordering::Relaxed) > next_score {
                will_die.store(max_node.game.you.dead(), Ordering::Relaxed);
                alpha_beta.new_beta_score(next_score);
            }
        };

        if parallel {
            let _res: Vec<()> = self.children.par_iter_mut().map(solver).collect();
        } else {
            let _res: Vec<()> = self.children.iter_mut().map(solver).collect();
        }

        if Instant::now() > *deadline {
            // deadline exceeded
            return (None, total_node_count.load(Ordering::Relaxed));
        }

        let min_score = min_score.load(Ordering::Relaxed);
        let min_score = if min_score == i64::MAX {
            None
        } else {
            Some(min_score)
        };
        self.will_die = will_die.load(Ordering::Relaxed);
        self.score = min_score;
        (min_score, total_node_count.load(Ordering::Relaxed))
    }
}
