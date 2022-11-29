use rayon::prelude::*;
use std::{
    cmp,
    fmt::Display,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
    time::Instant,
};

use super::{max::MaximizingNode, min::MinimizingNode};
use crate::logic::{Direction, Game};

pub struct AlphaBeta<'a, S: Ord + Clone> {
    parent: Option<&'a AlphaBeta<'a, S>>,
    alpha: RwLock<Option<S>>,
    beta: RwLock<Option<S>>,
}

impl<'a, S: Ord + Clone> AlphaBeta<'a, S> {
    pub fn new(a: Option<S>, b: Option<S>) -> Self {
        Self {
            parent: None,
            alpha: RwLock::new(a),
            beta: RwLock::new(b),
        }
    }

    fn new_child(&'a self) -> Self {
        Self {
            parent: Some(self),
            alpha: RwLock::new(self.alpha.read().unwrap().clone()),
            beta: RwLock::new(self.beta.read().unwrap().clone()),
        }
    }

    fn new_alpha_score(&self, a: S) {
        let next_score = Some(a);
        let new_alpha = *self.alpha.read().unwrap() < next_score;

        if new_alpha {
            let mut alpha_write = self.alpha.write().unwrap();
            if *alpha_write < next_score {
                *alpha_write = next_score;
            }
        }
    }

    fn new_beta_score(&self, b: S) {
        let new_beta = self
            .beta
            .read()
            .unwrap()
            .as_ref()
            .map_or(true, |old_b| *old_b > b);

        if new_beta {
            let mut beta_write = self.beta.write().unwrap();
            let next_score = Some(b);
            if *beta_write == None || *beta_write > next_score {
                *beta_write = next_score;
            }
        }
    }

    fn should_be_pruned(&self) -> bool {
        let mut max_alpha = self.alpha.read().unwrap().clone();
        let mut next = self;
        while let Some(v) = next.parent {
            let other_alpha = v.alpha.read().unwrap();
            if *other_alpha > max_alpha {
                max_alpha = other_alpha.clone();
            }
            next = v;
        }

        let mut next = self;
        loop {
            let beta = next.beta.read().unwrap();
            if *beta != None {
                if max_alpha > *beta {
                    return true;
                }
            }
            if let Some(p) = next.parent {
                next = p;
            } else {
                break;
            }
        }

        false
    }
}

impl<'a, S: Ord + Display + Clone + Send + Sync + 'static> MaximizingNode<S> {
    pub fn par_solve<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha_beta: &AlphaBeta<'_, S>,
        threads: f32,
    ) -> (Option<(Direction, S)>, usize)
    where
        FScore: Fn(&Game) -> S + Sync,
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
            self.game.execute_moves(Direction::Up, &vec![]);
            let score = score_fn(&self.game);
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

        let solver = |min_node: &mut MinimizingNode<S>| {
            if alpha_beta.should_be_pruned() {
                return;
            }

            let (next_score, node_count) = min_node.par_solve(
                game.clone(),
                deadline,
                max_depth,
                score_fn,
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
            }

            alpha_beta.new_alpha_score(next_score.unwrap());
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
        return (
            top_score.map(|s| (top_move, s)),
            total_node_count.load(Ordering::Relaxed),
        );
    }
}

impl<'a, S: Ord + Display + Clone + Sync + Send + 'static> MinimizingNode<S> {
    pub fn par_solve<FScore>(
        &mut self,
        game: Arc<&Game>,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha_beta: &AlphaBeta<'_, S>,
        threads: f32,
    ) -> (Option<S>, usize)
    where
        FScore: Fn(&Game) -> S + Sync,
    {
        let game = *game.as_ref();

        self.update_children(game);
        let (parallel, threads) = if threads > 1f32 {
            (true, threads / self.children.len() as f32)
        } else {
            (false, threads)
        };

        let min_score: RwLock<Option<S>> = RwLock::new(None);
        let alpha_beta = alpha_beta.new_child();
        let total_node_count = AtomicUsize::new(0);

        let solver = |max_node: &mut MaximizingNode<S>| {
            if alpha_beta.should_be_pruned() {
                return;
            }

            let (next_score, node_count) =
                max_node.par_solve(deadline, max_depth - 1, score_fn, &alpha_beta, threads);

            let next_score = if let Some(s) = next_score {
                s.1
            } else {
                return; // Deadline exceeded
            };

            total_node_count.fetch_add(node_count, Ordering::Relaxed);

            let mut min_score_write = min_score.write().unwrap();
            if *min_score_write != None {
                *min_score_write =
                    Some(cmp::min(min_score_write.as_ref().unwrap(), &next_score).clone());
            } else {
                *min_score_write = Some(next_score.clone());
            }

            alpha_beta.new_beta_score(next_score);
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

        let min_score = min_score.read().unwrap().clone();
        self.score = min_score.clone();
        (min_score, total_node_count.load(Ordering::Relaxed))
    }
}
