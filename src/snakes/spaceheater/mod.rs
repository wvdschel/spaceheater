use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{log, logic::Game, protocol, Battlesnake};

mod game_solver;
use game_solver::GameSolver;

mod scorecard;

#[derive(Clone)]
pub struct SpaceHeater<T>
where
    T: Ord + Default + Copy + Display + Send,
{
    last_turn_latency_estimate: Arc<AtomicU64>,
    recent_ping_average: Arc<AtomicU64>,
    score_fn: fn(&Game) -> T,
}

impl<T> SpaceHeater<T>
where
    T: Ord + Default + Copy + Display + Send,
{
    pub fn new(score_fn: fn(&Game) -> T) -> Self {
        Self {
            last_turn_latency_estimate: Arc::new(AtomicU64::new(100)),
            recent_ping_average: Arc::new(AtomicU64::new(100)),
            score_fn,
        }
    }

    fn calculate_latency(&self, last_turn_time_ms: u64, max_turn_time_ms: u64) -> Duration {
        let prev_latency_ms = self.last_turn_latency_estimate.load(Ordering::Acquire);
        let mut latency_ms = prev_latency_ms;
        if last_turn_time_ms > prev_latency_ms {
            let last_turn_compute_time_ms = max_turn_time_ms - prev_latency_ms;
            let last_turn_actual_latency = last_turn_time_ms - last_turn_compute_time_ms;

            let mut ping_avg = self.recent_ping_average.load(Ordering::Acquire);
            if last_turn_actual_latency > ping_avg {
                // Override ping average if we got a worse ping
                println!(
                    "Bad ping: {}ms (previous average: {}ms)",
                    last_turn_actual_latency, ping_avg
                );
                ping_avg = last_turn_actual_latency;
            } else {
                // Gradually decline if we got a better ping
                ping_avg = (ping_avg * 95 + last_turn_actual_latency * 5) / 100;
            }
            self.recent_ping_average.store(ping_avg, Ordering::Release);

            // 140% + 40ms seems like a sensible margin for ping fluctuations
            latency_ms = ping_avg * 14 / 10 + 40;
            log!("last turn took {}/{}ms, with {}ms slack for latency. Actual compute time {}, actual latency {}.",
                last_turn_time_ms, max_turn_time_ms, prev_latency_ms, last_turn_compute_time_ms, last_turn_actual_latency);

            if latency_ms > max_turn_time_ms {
                latency_ms = max_turn_time_ms * 10 / 20;
                log!(
                    "estimated latency exceeds turn time - limiting to {}ms",
                    latency_ms
                );
            }
        }
        self.last_turn_latency_estimate
            .store(latency_ms, Ordering::Release);
        Duration::from_millis(latency_ms)
    }
}

impl<T> Battlesnake for SpaceHeater<T>
where
    T: Ord + Default + Copy + Display + Send + 'static,
{
    fn snake_info(&self) -> protocol::SnakeInfo {
        let ping = self.recent_ping_average.load(Ordering::Relaxed);
        if ping < 500 {
            thread::sleep(Duration::from_millis(500 - ping));
        }
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "General Error".to_string(),
            color: "#E77200".to_string(),
            head: "workout".to_string(),
            tail: "flame".to_string(),
            version: "113".to_string(),
        }
    }

    fn start(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: &protocol::Request) -> Result<protocol::MoveResponse, String> {
        let last_turn_duration_ms = req.you.latency.parse::<u64>().unwrap_or(0);
        let max_turn_time_ms = req.game.timeout as u64;
        let latency = self.calculate_latency(last_turn_duration_ms, max_turn_time_ms);
        let start_time = Instant::now();
        let deadline = start_time + Duration::from_millis(max_turn_time_ms) - latency;

        println!(
            "----- request received at {:?}, latency {:?}, deadline set at {:?} -----",
            start_time, latency, deadline
        );
        let game = Game::from(req);
        let (best_dir, top_score) = GameSolver::new(self.score_fn).solve(&game, &deadline);

        println!(
            "----- Turn {}: top score {} when moving {} -----\n{}\n{}",
            req.turn,
            top_score,
            best_dir,
            &game,
            std::iter::repeat("-").take(100).collect::<String>(),
        );

        println!("deadline: {:?}, now: {:?}", deadline, Instant::now());
        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: "".to_string(),
        })
    }
}
