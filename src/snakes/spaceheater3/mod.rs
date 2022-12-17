use crate::{
    log,
    logic::{self, Game},
    protocol::{self, Customizations, Direction},
    util::thread_count,
    Battlesnake,
};

use std::{
    cmp,
    collections::HashMap,
    sync::{atomic::AtomicBool, mpsc::channel, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use self::{background::BackgroundWorker, max::MaximizingNode};

pub mod alphabeta;
mod background;
pub mod max;
pub mod min;
mod util;

pub const DEFAULT_COLOR: &str = "#b54d47";
pub const DEFAULT_HEAD: &str = "scarf";
pub const DEFAULT_TAIL: &str = "rocket";
const LATENCY_MARGIN: Duration = Duration::from_millis(100);

pub struct Spaceheater3<S>
where
    S: logic::scoring::Scorer + Sync + Clone,
{
    scorer: S,
    customizations: Customizations,
    background_workers: Arc<Mutex<HashMap<String, background::BackgroundWorker>>>,
}

impl<S> Spaceheater3<S>
where
    S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
{
    pub fn new(scorer: S, customizations: Option<Customizations>) -> Self {
        Self {
            scorer,
            customizations: customizations.unwrap_or(Customizations {
                color: DEFAULT_COLOR.into(),
                head: DEFAULT_HEAD.into(),
                tail: DEFAULT_TAIL.into(),
            }),
            background_workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_background_work(&self, game_id: &String, g: Game) -> MaximizingNode {
        let workers = self.background_workers.lock().unwrap();

        let bgworker = if let Some(v) = workers.get(game_id) {
            v
        } else {
            log!("no background work found for {}", game_id);
            return MaximizingNode::new(g.clone());
        };

        let res = bgworker.foreground(g);
        log!(
            "foregrounding work for {} at depth {}",
            game_id,
            res.depth_completed
        );
        res
    }

    pub fn solve(
        &self,
        game: Game,
        deadline: &Instant,
        game_id: String,
    ) -> Option<(Direction, i64)> {
        let _turn = game.turn;
        let _start = Instant::now();

        let (tx, rx) = channel();

        let scorer = self.scorer.clone();
        let deadline = deadline.clone();
        let enemy_count = game.others.len();
        let mut root = self.get_background_work(&game_id, game);
        let base_depth = cmp::max(root.depth_completed, base_depth(enemy_count));
        let bgworkers = self.background_workers.clone();

        log!(
            "turn {}: start: calculating depths {} through ... using {} threads",
            _turn,
            base_depth,
            thread_count(),
        );
        thread::spawn(move || {
            let mut best_score = root.score;
            let mut last_score = None;
            let mut _total_node_count = 0;
            for current_depth in base_depth..usize::MAX {
                log!(
                    "turn {}: {}ms: starting depth {}",
                    _turn,
                    _start.elapsed().as_millis(),
                    current_depth,
                );
                let (res, node_count) = root.solve(
                    Arc::new(AtomicBool::new(false)),
                    &deadline,
                    current_depth,
                    &scorer,
                    &alphabeta::AlphaBeta::new(i64::MIN, i64::MAX),
                    thread_count() as f32,
                );
                _total_node_count += node_count;
                //log!("complete tree for depth {}:\n{}", current_depth, root);

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
            let workers = bgworkers.lock().unwrap();
            let bgworker = if let Some(v) = workers.get(&game_id) {
                v
            } else {
                return;
            };

            for c in root.children {
                if let Some(best_dir) = best_score.map(|s| s.0) {
                    if c.my_move == best_dir {
                        log!(
                            "backgrounding work for {} at depth {}",
                            game_id,
                            c.depth_completed
                        );
                        bgworker.background(background::BackgroundWork::Min(root.game.clone(), c))
                    }
                }
            }
        });

        rx.recv().unwrap()
    }
}

impl<S> Battlesnake for Spaceheater3<S>
where
    S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
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

    fn start(&self, req: &crate::protocol::Request) -> Result<(), String> {
        #[cfg(not(feature = "sequential"))]
        {
            let game_id = req.game.id.clone();
            let game = Game::from(req);
            let work = background::BackgroundWork::Max(MaximizingNode::new(game));
            let worker = BackgroundWorker::new(self.scorer.clone());
            worker.background(work);

            let mut workers = self.background_workers.lock().unwrap();
            workers.insert(game_id, worker);
        }

        Ok(())
    }

    fn end(&self, req: &crate::protocol::Request) -> Result<(), String> {
        let workers = self.background_workers.lock().unwrap();
        if let Some(w) = workers.get(&req.game.id) {
            w.cancel();
        }

        Ok(())
    }

    fn make_move(
        &self,
        req: &crate::protocol::Request,
    ) -> Result<crate::protocol::MoveResponse, String> {
        let game_id = req.game.id.clone();
        let game = Game::from(req);
        let deadline = Instant::now() + game.timeout - LATENCY_MARGIN;
        let res = self.solve(game, &deadline, game_id);

        let (best_dir, top_score) = res
            .map(|(dir, score)| (dir, format!("{}", score)))
            .unwrap_or((Direction::Up, "no result".to_string()));

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: top_score,
        })
    }
}

fn base_depth(enemy_count: usize) -> usize {
    match enemy_count {
        0 => 5,
        1 => 3,
        2 => 2,
        3 => 2,
        4 => 2,
        _ => 1,
    }
}
