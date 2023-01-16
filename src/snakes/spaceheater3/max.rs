use rayon::prelude::*;

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};

use crate::{
    logic::{self, Direction, Game},
    protocol::ALL_DIRECTIONS,
};

use super::{alphabeta::AlphaBeta, min::MinimizingNode, util::certain_death};

pub struct MaximizingNode {
    pub(super) game: Game,
    pub(super) score: Option<(Direction, i64)>,
    pub(super) children: Vec<MinimizingNode>,
    pub(super) will_die: bool,
}

impl MaximizingNode {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            score: None,
            children: vec![],
            will_die: false,
        }
    }

    fn update_children(&mut self) {
        if self.children.len() == 0 {
            for my_dir in ALL_DIRECTIONS {
                let mut my_pos = self.game.you.head.neighbour(my_dir);
                self.game.warp(&mut my_pos);
                if !certain_death(&self.game, &self.game.you, &my_pos) {
                    self.children.push(MinimizingNode::new(my_dir));
                }
            }
        } else {
            self.children.sort_unstable_by(|c1, c2| c1.cmp_scores(c2));
        }
    }

    fn check_bounds<S>(&mut self, max_depth: usize, scorer: &S) -> bool
    where
        S: logic::scoring::Scorer,
    {
        if self.game.you.dead() {
            if self.score == None {
                self.score = Some((Direction::Up, scorer.score(&self.game)));
            }
            self.will_die = true;
            return true;
        }
        if max_depth == 0 {
            let score = scorer.score(&self.game);
            self.score = Some((Direction::Up, score));
            return true;
        }

        false
    }
}

impl MaximizingNode {
    pub fn solve<S>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        scorer: &S,
        alpha_beta: &AlphaBeta<'_>,
        threads: f32,
    ) -> (Option<(Direction, i64)>, usize)
    where
        S: logic::scoring::Scorer + Sync + Clone + 'static,
    {
        if Instant::now() > *deadline {
            return (None, 0);
        }
        if self.check_bounds(max_depth, scorer) {
            return (self.score.clone(), 1);
        }
        self.update_children();

        if self.children.len() == 0 {
            // All paths are certain death, just score this board and return
            self.game.execute_moves(Direction::Up, &vec![]);
            let score = scorer.score(&self.game);
            self.score = Some((Direction::Up, score));
            return (self.score.clone(), 1);
        }

        let (parallel, threads) = if threads > 1f32 {
            (true, threads / self.children.len() as f32)
        } else {
            (false, threads)
        };

        let game = Arc::new(&self.game);
        let top_score = RwLock::new((Direction::Up, None));
        let alpha_beta = alpha_beta.new_child();
        let total_node_count = AtomicUsize::new(0);
        let will_die = AtomicBool::new(false);

        let solver = |min_node: &mut MinimizingNode| {
            if alpha_beta.should_be_pruned() {
                return;
            }

            let (next_score, node_count) = min_node.solve(
                game.clone(),
                deadline,
                max_depth,
                scorer,
                &alpha_beta,
                threads,
            );
            total_node_count.fetch_add(node_count, Ordering::Relaxed);

            if next_score == None {
                return; // Deadline exceeded
            }

            let top_score_read = top_score.read().unwrap();
            let new_max_score = top_score_read.1 < next_score;
            drop(top_score_read);

            if new_max_score {
                let mut top_score_write = top_score.write().unwrap();
                if top_score_write.1 < next_score {
                    *top_score_write = (min_node.my_move, next_score.clone());
                    will_die.store(min_node.will_die, Ordering::Relaxed);
                }
                alpha_beta.new_alpha_score(next_score.unwrap());
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

        let (top_move, top_score) = top_score.read().unwrap().clone();
        self.score = top_score.map(|s| (top_move, s));
        self.will_die = will_die.load(Ordering::Relaxed);
        return (self.score, total_node_count.load(Ordering::Relaxed));
    }

    pub fn cmp_scores(&self, other: &Self) -> std::cmp::Ordering {
        let self_score = self.score.map(|s| s.1).unwrap_or(i64::MAX);
        let other_score = other.score.map(|s| s.1).unwrap_or(i64::MAX);
        self_score.cmp(&other_score)
    }
}

impl std::fmt::Display for MaximizingNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "will_die = {}: {}\n",
            self.will_die, self.game
        ))?;
        if let Some(max_choice) =
            self.children
                .iter()
                .reduce(|max, child| if child.score > max.score { child } else { max })
        {
            f.write_fmt(format_args!(
                "turn {}: picking {} with score {} (will_die = {})\n",
                self.game.turn,
                max_choice.my_move,
                max_choice.score.unwrap_or(0),
                max_choice.will_die,
            ))?;
            return max_choice.fmt(f);
        }
        Ok(())
    }
}
