use std::{
    cmp,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{log, logic, logic::Game, util::thread_count};

use super::{alphabeta, base_depth, max::MaximizingNode, min::MinimizingNode};

pub(super) enum BackgroundWork {
    Max(MaximizingNode),
    Min(Game, MinimizingNode),
}

pub(super) struct BackgroundWorker {
    background_sender: Sender<BackgroundWork>,
    foreground_sender: Sender<Option<Game>>,
    foreground_receiver: Receiver<Option<MaximizingNode>>,
}

impl BackgroundWorker {
    pub fn new<S>(scorer: S) -> Self
    where
        S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    {
        let (bg_tx, bg_rx) = channel();
        let (fg_tx, fg_rx) = channel();
        let (fg_res_tx, fg_res_rx) = channel();

        thread::spawn(move || background_fn(scorer, bg_rx, fg_rx, fg_res_tx));

        Self {
            background_sender: bg_tx,
            foreground_sender: fg_tx,
            foreground_receiver: fg_res_rx,
        }
    }

    pub fn cancel(&self) {
        if let Err(e) = self.foreground_sender.send(None) {
            println!("failed to cancel background computation: {}", e)
        }
    }

    pub fn background(&self, w: BackgroundWork) {
        if let Err(e) = self.background_sender.send(w) {
            println!("failed to initiate background computation: {}", e)
        }
    }

    pub fn foreground(&self, game: Game) -> MaximizingNode {
        if let Err(e) = self.foreground_sender.send(Some(game.clone())) {
            println!("failed to send to background worker: {}", e);
            return MaximizingNode::new(game);
        }

        match self.foreground_receiver.recv() {
            Ok(node) => match node {
                Some(max) => max,
                None => {
                    log!("no background work found");
                    MaximizingNode::new(game)
                }
            },
            Err(e) => {
                println!("failed to receive from background worker: {}", e);
                MaximizingNode::new(game)
            }
        }
    }
}

fn background_fn<S>(
    scorer: S,
    bg_rx: Receiver<BackgroundWork>,
    fg_rx: Receiver<Option<Game>>,
    fg_res_tx: Sender<Option<MaximizingNode>>,
) -> ()
where
    S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
{
    loop {
        let mut work = match bg_rx.recv() {
            Ok(n) => n,
            Err(_) => break,
        };

        let cancelled = Arc::new(AtomicBool::new(false));
        let join_handle = {
            let cancelled = cancelled.clone();
            let scorer = scorer.clone();
            let deadline = Instant::now() + Duration::from_millis(2000);
            thread::spawn(move || {
                work.solve(cancelled, &deadline, scorer);
                work
            })
        };

        let fg_cmd = fg_rx.recv();
        cancelled.store(true, Ordering::Relaxed);
        match fg_cmd {
            Ok(g) => match g {
                Some(g) => {
                    let work = join_handle.join().unwrap();
                    fg_res_tx.send(work.result(g)).unwrap_or(())
                }
                None => fg_res_tx.send(None).unwrap_or(()),
            },
            Err(e) => {
                println!("stopping background worker: {}", e);
                return;
            }
        };
    }
}

impl BackgroundWork {
    fn solve<S>(&mut self, cancelled: Arc<AtomicBool>, deadline: &Instant, scorer: S)
    where
        S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    {
        while !cancelled.load(Ordering::Relaxed) && Instant::now() < *deadline {
            match self {
                BackgroundWork::Max(max) => {
                    let next_depth =
                        cmp::max(max.depth_completed + 1, base_depth(max.game.others.len())); // TODO
                    log!("starting depth {} in the background", next_depth);
                    max.solve(
                        cancelled.clone(),
                        &deadline,
                        next_depth,
                        &scorer,
                        &alphabeta::AlphaBeta::new(i64::MIN, i64::MAX),
                        thread_count() as f32,
                    );
                }
                BackgroundWork::Min(game, min) => {
                    let next_depth =
                        cmp::max(min.depth_completed + 1, base_depth(game.others.len())); // TODO
                    log!("starting depth {} in the background", next_depth);
                    let game_arc = Arc::new(game as &Game);
                    min.solve(
                        game_arc,
                        cancelled.clone(),
                        &deadline,
                        next_depth,
                        &scorer,
                        &alphabeta::AlphaBeta::new(i64::MIN, i64::MAX),
                        thread_count() as f32,
                    );
                }
            }
        }
    }

    fn result(self, game: Game) -> Option<MaximizingNode> {
        match self {
            BackgroundWork::Max(m) => {
                if game == m.game {
                    Some(m)
                } else {
                    None
                }
            }
            BackgroundWork::Min(_, m) => {
                for c in m.children {
                    if c.game == game {
                        return Some(c);
                    }
                }
                None
            }
        }
    }
}
