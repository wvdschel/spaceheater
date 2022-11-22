use std::{
    cmp,
    fmt::Display,
    sync::{mpsc::Receiver, Arc},
    time::Instant,
};

use crate::{
    logic::{Direction, Game},
    util::threadpool::Threadpool,
};

use super::{max::MaximizingNode, min::MinimizingNode};

pub struct WorkItem<S, Fscore>
where
    S: Ord + Display + Clone + Send + 'static,
    Fscore: Fn(&Game) -> S + Send + 'static,
{
    node: Node<S>,
    deadline: Instant,
    max_depth: usize,
    score_fn: Fscore,
    alpha: Option<S>,
    beta: Option<S>,
    cancelled: Receiver<()>,
}

pub enum Node<S>
where
    S: Ord + Display + Clone + Send + 'static,
{
    Min(MinimizingNode<S>, Arc<Game>),
    Max(MaximizingNode<S>),
}

pub fn do_work<S, Fscore>(work: WorkItem<S, Fscore>) -> WorkItem<S, Fscore>
where
    S: Ord + Display + Clone + Send + 'static,
    Fscore: Fn(&Game) -> S + Send + 'static,
{
    let mut work = work;
    work.node = match work.node {
        Node::Min(mut min, game) => {
            min.solve(
                game.as_ref(),
                &work.deadline,
                work.max_depth,
                &work.score_fn,
                work.alpha.clone(),
                work.beta.clone(),
            );
            Node::Min(min, game)
        }
        Node::Max(mut max) => {
            max.solve(
                &work.deadline,
                work.max_depth,
                &work.score_fn,
                work.alpha.clone(),
                work.beta.clone(),
            );
            Node::Max(max)
        }
    };

    return work;
}

impl<'a, S: Ord + Display + Clone + Send + 'static> MaximizingNode<S> {
    pub fn solve_concurrent<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
        threadpool: Threadpool<WorkItem<S, FScore>, WorkItem<S, FScore>>,
        thread_count: f64,
    ) -> (Option<(Direction, S)>, usize)
    where
        FScore: Fn(&Game) -> S + Send,
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

        if self.children.len() as f64 > thread_count {
            return self.fork(deadline, max_depth, score_fn, alpha, beta, threadpool);
        }

        let mut best_dir = Direction::Up;
        let mut max_score = None;
        let mut alpha = alpha;
        let mut total_node_count = 0;
        let threads_per_child = thread_count / self.children.len() as f64;
        for min_node in &mut self.children {
            let (next_score, node_count) = min_node.solve_concurrent(
                &mut self.game,
                deadline,
                max_depth,
                score_fn,
                alpha.clone(),
                beta.clone(),
                threadpool,
                threads_per_child,
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

    fn fork<FScore>(
        &mut self,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
        threadpool: Threadpool<WorkItem<S, FScore>, WorkItem<S, FScore>>,
    ) -> (Option<(Direction, S)>, usize)
    where
        FScore: Fn(&Game) -> S + Send,
    {
        todo!()
    }
}

impl<'a, S: Ord + Display + Clone + Send + 'static> MinimizingNode<S> {
    pub fn solve_concurrent<FScore>(
        &mut self,
        game: &Game,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
        threadpool: Threadpool<WorkItem<S, FScore>, WorkItem<S, FScore>>,
        thread_count: f64,
    ) -> (Option<S>, usize)
    where
        FScore: Fn(&Game) -> S + Send,
    {
        self.update_children(game);

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

    fn fork<FScore>(
        &mut self,
        game: &Game,
        deadline: &Instant,
        max_depth: usize,
        score_fn: &FScore,
        alpha: Option<S>,
        beta: Option<S>,
        threadpool: Threadpool<WorkItem<S, FScore>, WorkItem<S, FScore>>,
    ) -> (Option<S>, usize)
    where
        FScore: Fn(&Game) -> S + Send,
    {
        todo!()
    }
}
