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

use crate::{
    logic::{Direction, Game},
    protocol::ALL_DIRECTIONS,
};

use super::{alphabeta::AlphaBeta, min::MinimizingNode, util::certain_death};

pub struct MaximizingNode<S: Ord + Display + Clone + 'static> {
    pub(super) game: Game,
    pub(super) score: Option<(Direction, S)>,
    pub(super) children: Vec<MinimizingNode<S>>,
}

impl<'a, S: Ord + Display + Clone + 'static> MaximizingNode<S> {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            score: None,
            children: vec![],
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
            self.children.sort_unstable_by(|c1, c2| c1.cmp_scores(c2))
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

impl<'a, S: Ord + Display + Clone + Send + Sync + 'static> MaximizingNode<S> {
    pub fn solve<FScore>(
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

            let (next_score, node_count) = min_node.solve(
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

impl<'a, S: Ord + Display + Clone + 'static> std::fmt::Display for MaximizingNode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.format_tree(0).as_str())
    }
}
