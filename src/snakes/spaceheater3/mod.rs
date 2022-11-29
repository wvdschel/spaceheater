use crate::{
    log,
    logic::Game,
    protocol::{self, Customizations, Direction},
    snakes::spaceheater3::max::MaximizingNode,
    util::thread_count,
    Battlesnake,
};
use std::{
    cmp,
    fmt::Display,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

pub mod max;
pub mod min;
pub mod parallel;
mod util;

pub const DEFAULT_COLOR: &str = "#b54d47";
pub const DEFAULT_HEAD: &str = "scarf";
pub const DEFAULT_TAIL: &str = "rocket";
const LATENCY_MARGIN: Duration = Duration::from_millis(100);

pub struct Spaceheater3<Fscore, S>
where
    Fscore: Fn(&Game) -> S + Sync + Clone + 'static,
    S: Ord + Display + Clone + Sync + Send + 'static,
{
    score_fn: Fscore,
    customizations: Customizations,
    parallel: bool,
}

impl<Fscore, S> Spaceheater3<Fscore, S>
where
    Fscore: Fn(&Game) -> S + Sync + Send + Clone + 'static,
    S: Ord + Display + Clone + Sync + Send + 'static,
{
    pub fn new(score_fn: Fscore, customizations: Option<Customizations>, parallel: bool) -> Self {
        Self {
            score_fn,
            customizations: customizations.unwrap_or(Customizations {
                color: DEFAULT_COLOR.into(),
                head: DEFAULT_HEAD.into(),
                tail: DEFAULT_TAIL.into(),
            }),
            parallel,
        }
    }

    pub fn solve(
        &self,
        game: Game,
        deadline: &Instant,
        max_depth: usize,
    ) -> Option<(Direction, S)> {
        let enemy_count = game.others.len();
        let turn = game.turn;

        let base_depth = match enemy_count {
            0 => 5,
            1 => 3,
            2 => 2,
            3 => 2,
            4 => 2,
            _ => 1,
        };
        let start = Instant::now();
        let max_depth = cmp::max(base_depth + 1, max_depth);

        println!(
            "turn {}: start: calculating depths {} through {}",
            turn, base_depth, max_depth
        );

        let mut best_score = None;
        let mut last_score = None;
        for current_depth in base_depth..max_depth {
            println!(
                "turn {}: {}ms: starting depth {}",
                turn,
                start.elapsed().as_millis(),
                current_depth,
            );
            let (tx, rx) = channel();
            {
                let score_fn = self.score_fn.clone();
                let parallel = self.parallel;
                let deadline = deadline.clone();
                let game = game.clone();
                thread::spawn(move || {
                    let mut root = MaximizingNode::new(game);
                    let (res, node_count) = if parallel {
                        root.par_solve(
                            &deadline,
                            current_depth,
                            &score_fn,
                            &parallel::AlphaBeta::new(None, None),
                            thread_count() as f32,
                        )
                    } else {
                        root.solve(&deadline, current_depth, &score_fn, None, None)
                    };
                    let _ = tx.send((res, node_count));
                    log!("complete tree for depth {}:\n{}", current_depth, root);
                });
            }
            let (res, node_count) = rx.recv().unwrap();
            match &res {
                Some((dir, score)) => {
                    best_score = res.clone();
                    println!(
                        "turn {}: {}ms: completed depth {}, tree has {} nodes: {} {}",
                        turn,
                        start.elapsed().as_millis(),
                        current_depth,
                        node_count,
                        dir,
                        score,
                    );
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
            if last_score == best_score.as_ref().map(|s| s.1.clone()) {
                println!(
                    "turn {}: {}ms: tree completed at depth {}",
                    turn,
                    start.elapsed().as_millis(),
                    current_depth - 1,
                );
                break;
            }
            last_score = best_score.as_ref().map(|s| s.1.clone())
        }

        let statm = procinfo::pid::statm_self().unwrap();
        println!(
            "turn {}: {}ms / {} MB: returning {}",
            turn,
            start.elapsed().as_millis(),
            statm.size * 4096 / 1024 / 1024,
            best_score
                .clone()
                .map(|v| v.0.to_string())
                .unwrap_or("None".to_string())
        );

        best_score
    }
}

impl<Fscore, S> Battlesnake for Spaceheater3<Fscore, S>
where
    Fscore: Fn(&Game) -> S + Sync + Send + Clone + 'static,
    S: Ord + Display + Clone + Send + Sync,
{
    fn snake_info(&self) -> crate::protocol::SnakeInfo {
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "".to_string(),
            color: self.customizations.color.clone(),
            head: self.customizations.head.clone(),
            tail: self.customizations.tail.clone(),
            version: "2".to_string(),
        }
    }

    fn start(&self, _: &crate::protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: &crate::protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(
        &self,
        req: &crate::protocol::Request,
    ) -> Result<crate::protocol::MoveResponse, String> {
        let game = Game::from(req);
        let deadline = Instant::now() + game.timeout - LATENCY_MARGIN;
        let res = self.solve(game, &deadline, usize::MAX);

        let (best_dir, top_score) = res
            .map(|(dir, score)| (dir, format!("{}", score)))
            .unwrap_or((Direction::Up, "no result".to_string()));

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: top_score,
        })
    }
}
