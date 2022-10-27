use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use crate::{
    logic::Game,
    protocol::{self, Customizations},
    Battlesnake,
};

mod game_solver;
pub use game_solver::GameSolver;

mod scorecard;

#[derive(Clone)]
pub struct SpaceHeater<T>
where
    T: Ord + Default + Copy + Display + Send,
{
    score_fn: fn(&Game) -> T,
    customizations: Customizations,
}

impl<T> SpaceHeater<T>
where
    T: Ord + Default + Copy + Display + Send,
{
    pub fn new(score_fn: fn(&Game) -> T, customizations: Customizations) -> Self {
        Self {
            score_fn,
            customizations,
        }
    }
}

impl<T> Battlesnake for SpaceHeater<T>
where
    T: Ord + Default + Copy + Display + Send + 'static,
{
    fn snake_info(&self) -> protocol::SnakeInfo {
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "".to_string(),
            color: self.customizations.color.clone(),
            head: self.customizations.head.clone(),
            tail: self.customizations.tail.clone(),
            version: "0".to_string(),
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
        let start_time = Instant::now();
        let deadline =
            start_time + Duration::from_millis(max_turn_time_ms) - Duration::from_millis(35);

        println!(
            "----- request received at {:?}, latency {}ms, deadline set at {:?} -----",
            start_time, last_turn_duration_ms, deadline
        );
        let game = Game::from(req);
        let (best_dir, top_score) =
            GameSolver::new(self.score_fn).solve("", &game, Some(&deadline), usize::MAX);

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
