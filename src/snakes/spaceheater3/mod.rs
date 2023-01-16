use crate::{
    log,
    logic::{self, Game},
    protocol::{self, Customizations, Direction},
    snakes::{spaceheater3::max::MaximizingNode, Salami},
    util::thread_count,
    Battlesnake,
};
use std::{
    cmp,
    sync::{mpsc::channel, Arc},
    thread,
    time::{Duration, Instant},
};

pub mod alphabeta;
pub mod max;
pub mod min;
pub mod util;

pub const DEFAULT_COLOR: &str = "#b54d47";
pub const DEFAULT_HEAD: &str = "scarf";
pub const DEFAULT_TAIL: &str = "rocket";
const LATENCY_MARGIN: Duration = Duration::from_millis(100);

pub struct Spaceheater3<S, S2>
where
    S: logic::scoring::Scorer + Sync + Clone,
    S2: logic::scoring::Scorer + Sync + Clone,
{
    scorer: S,
    customizations: Customizations,
    fallback: Arc<Salami<S2>>,
}

impl<S, S2> Spaceheater3<S, S2>
where
    S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    S2: logic::scoring::Scorer + Send + Sync + Clone + 'static,
{
    pub fn new(scorer: S, fallback_scorer: S2, customizations: Option<Customizations>) -> Self {
        Self {
            scorer,
            customizations: customizations.unwrap_or(Customizations {
                color: DEFAULT_COLOR.into(),
                head: DEFAULT_HEAD.into(),
                tail: DEFAULT_TAIL.into(),
            }),
            fallback: Arc::new(Salami::new(fallback_scorer, None)),
        }
    }

    pub fn solve(
        &self,
        game: Game,
        deadline: &Instant,
        max_depth: usize,
    ) -> Option<(Direction, i64)> {
        let enemy_count = game.others.len();
        let _turn = game.turn;

        let base_depth = match enemy_count {
            0 => 5,
            1 => 3,
            2 => 2,
            3 => 2,
            4 => 2,
            _ => 1,
        };
        let _start = Instant::now();
        let max_depth = cmp::max(base_depth + 1, max_depth);

        log!(
            "turn {}: start: calculating depths {} through {} using {} threads",
            _turn,
            base_depth,
            max_depth,
            thread_count(),
        );

        let (tx, rx) = channel();

        let scorer = self.scorer.clone();
        let deadline = deadline.clone();
        let game = game.clone();
        let fallback = self.fallback.clone();
        thread::spawn(move || {
            let mut root = MaximizingNode::new(game.clone());
            let mut best_score = None;
            let mut last_score = None;
            let mut _total_node_count = 0;
            for current_depth in base_depth..max_depth {
                log!(
                    "turn {}: {}ms: starting depth {}",
                    _turn,
                    _start.elapsed().as_millis(),
                    current_depth,
                );
                let (res, node_count) = root.solve(
                    &deadline,
                    current_depth,
                    &scorer,
                    &alphabeta::AlphaBeta::new(i64::MIN, i64::MAX),
                    thread_count() as f32,
                );
                _total_node_count += node_count;
                log!(
                    "complete min-max path for depth {}:\n{}",
                    current_depth,
                    root
                );

                match &res {
                    Some((_dir, _score)) => {
                        best_score = res.clone();
                        log!(
                            "turn {}: {}ms: completed depth {}, tree has {} nodes: {} {}",
                            _turn,
                            _start.elapsed().as_millis(),
                            current_depth,
                            _total_node_count,
                            _dir,
                            _score,
                        );
                    }
                    None => {
                        log!(
                            "turn {}: {}ms: aborted depth {}",
                            _turn,
                            _start.elapsed().as_millis(),
                            current_depth
                        );
                        break;
                    }
                }
                if last_score == best_score.as_ref().map(|s| s.1.clone()) {
                    log!(
                        "turn {}: {}ms: tree completed at depth {} after {} nodes",
                        _turn,
                        _start.elapsed().as_millis(),
                        current_depth - 1,
                        _total_node_count,
                    );
                    break;
                }
                last_score = best_score.as_ref().map(|s| s.1.clone())
            }

            if root.will_die {
                //println!("minimax thinks we will die, go into avoidance mode (monte carlo)");
                //let deadline = deadline + LATENCY_MARGIN / 5;
                //best_score = Some(fallback.solve(game, &deadline));
            }

            let _ = tx.send(best_score);
            let _statm = procinfo::pid::statm_self().unwrap();
            log!(
                "turn {}: {}ms / {} MB: returning {}",
                _turn,
                _start.elapsed().as_millis(),
                _statm.size * 4096 / 1024 / 1024,
                best_score
                    .clone()
                    .map(|v| v.0.to_string())
                    .unwrap_or("None".to_string())
            );
            log!("scores for moves: ");
            for _c in &root.children {
                log!(
                    "{}: {}",
                    _c.my_move,
                    _c.score
                        .map(|s| s.to_string())
                        .unwrap_or("pruned".to_string())
                )
            }
        });

        rx.recv().unwrap()
    }
}

impl<S, S2> Battlesnake for Spaceheater3<S, S2>
where
    S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    S2: logic::scoring::Scorer + Send + Sync + Clone + 'static,
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
        let start = Instant::now();
        let deadline = start + game.timeout - LATENCY_MARGIN;
        let res = self.solve(game, &deadline, usize::MAX);

        let (best_dir, top_score) = res
            .map(|(dir, score)| (dir, format!("{}", score)))
            .unwrap_or((Direction::Up, "no result".to_string()));

        if Instant::now() >= deadline + LATENCY_MARGIN / 2 {
            println!(
                "Warning: exceeded deadline: move took {}ms",
                start.elapsed().as_millis()
            )
        }

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: top_score,
        })
    }
}
