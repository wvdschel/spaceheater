use crate::{
    logic::Game,
    protocol::{self, Customizations, Direction},
    Battlesnake,
};
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

mod max;
mod min;
pub mod solve;
mod util;

pub const DEFAULT_COLOR: &str = "#b54d47";
pub const DEFAULT_HEAD: &str = "scarf";
pub const DEFAULT_TAIL: &str = "rocket";
const LATENCY_MARGIN: Duration = Duration::from_millis(120);

pub struct Spaceheater3<Fscore, S>
where
    Fscore: Fn(&Game) -> S,
    S: Ord + Display + Clone + Send + 'static,
{
    score_fn: Fscore,
    customizations: Customizations,
}

impl<Fscore, S> Spaceheater3<Fscore, S>
where
    Fscore: Fn(&Game) -> S,
    S: Ord + Display + Clone + Send + 'static,
{
    pub fn new(score_fn: Fscore, customizations: Option<Customizations>) -> Self {
        Self {
            score_fn,
            customizations: customizations.unwrap_or(Customizations {
                color: DEFAULT_COLOR.into(),
                head: DEFAULT_HEAD.into(),
                tail: DEFAULT_TAIL.into(),
            }),
        }
    }

    fn solve(&self, game: &Game, deadline: &Instant, max_depth: usize) -> Option<(Direction, S)> {
        solve::solve(game.clone(), deadline, max_depth, &self.score_fn)
    }
}

impl<Fscore, S> Battlesnake for Spaceheater3<Fscore, S>
where
    Fscore: Fn(&Game) -> S,
    S: Ord + Display + Clone + Send,
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
        let res = self.solve(&game, &deadline, usize::MAX);

        let (best_dir, top_score) = res
            .map(|(dir, score)| (dir, format!("{}", score)))
            .unwrap_or((Direction::Up, "no result".to_string()));

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: top_score,
        })
    }
}
