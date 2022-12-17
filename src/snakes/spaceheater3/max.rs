use rayon::prelude::*;

use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
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
    will_die: bool,
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
            self.will_die = true;
            if self.score == None {
                self.score = Some((Direction::Up, scorer.score(&self.game)));
            }
            return true;
        }
        if max_depth == 0 {
            let score = scorer.score(&self.game);
            self.score = Some((Direction::Up, score));
            return true;
        }

        false
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

        for c in self.children.iter() {
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

    pub(crate) fn will_die(&self) -> bool {
        self.will_die
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
                }
                will_die.store(min_node.will_die(), Ordering::Relaxed);
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
        self.will_die = will_die.load(Ordering::Relaxed);
        self.score = top_score.map(|s| (top_move, s));
        return (self.score, total_node_count.load(Ordering::Relaxed));
    }

    pub fn cmp_scores(&self, other: &Self) -> std::cmp::Ordering {
        let self_score = self.score.map(|s| s.1).unwrap_or(i64::MAX);
        let other_score = other.score.map(|s| s.1).unwrap_or(i64::MAX);
        self_score.cmp(&other_score)
    }

    pub fn solve_optimistic<S>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        scorer: &S,
        threads: f32,
    ) -> (Direction, i64)
    where
        S: logic::scoring::Scorer + Sync + Clone + 'static,
    {
        if max_depth == 0 {
            if self.score.is_none() {
                let score = scorer.score(&self.game);
                self.score = Some((Direction::Up, score));
            }
            return self.score.unwrap();
        }

        self.update_children();

        let (parallel, threads) = if threads > 1f32 {
            (true, threads / self.children.len() as f32)
        } else {
            (false, threads)
        };

        let top_score = Mutex::new((Direction::Up, i64::MIN));
        let game = Arc::new(&self.game);
        let solver = |min_node: &mut MinimizingNode| {
            if Instant::now() > *deadline {
                return;
            }
            let next_score =
                min_node.solve_optimistic(game.clone(), deadline, max_depth, scorer, threads);

            let mut best_score = top_score.lock().unwrap();
            if best_score.1 < next_score {
                *best_score = (min_node.my_move, next_score);
            }
        };

        if parallel {
            let _res: Vec<()> = self.children.par_iter_mut().map(solver).collect();
        } else {
            let _res: Vec<()> = self.children.iter_mut().map(solver).collect();
        }

        let res = top_score.lock().unwrap().clone();
        res
    }
}

impl std::fmt::Display for MaximizingNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_tree(0).as_str())
    }
}
