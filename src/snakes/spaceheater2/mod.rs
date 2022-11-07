mod scores;
mod solve;
mod util;

use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use protocol::Direction;

use crate::{
    logic::Game,
    protocol::{self, Customizations, ALL_DIRECTIONS},
    Battlesnake,
};

use self::util::certain_death;

pub const DEFAULT_COLOR: &str = "#b54d47";
pub const DEFAULT_HEAD: &str = "scarf";
pub const DEFAULT_TAIL: &str = "rocket";
const LATENCY_MARGIN: Duration = Duration::from_millis(130);

pub struct Spaceheater2<Fscore, Fmin, Fmax, S1, S2, S3>
where
    Fscore: Fn(&Game) -> S1,
    Fmin: Fn(&Game) -> S2,
    Fmax: Fn(&Game) -> S3,
    S1: Ord + Display + Clone + Send + 'static,
    S2: Ord + PartialOrd<S1>,
    S3: Ord + PartialOrd<S1>,
{
    expensive_score_fn: Fscore,
    cheap_min_score_fn: Fmin,
    cheap_max_score_fn: Fmax,
    customizations: Customizations,
}

impl<Fscore, Fmin, Fmax, S1, S2, S3> Spaceheater2<Fscore, Fmin, Fmax, S1, S2, S3>
where
    Fscore: Fn(&Game) -> S1,
    Fmin: Fn(&Game) -> S2,
    Fmax: Fn(&Game) -> S3,
    S1: Ord + Display + Clone + Send,
    S2: Ord + PartialOrd<S1>,
    S3: Ord + PartialOrd<S1>,
{
    pub fn new(
        expensive_score_fn: Fscore,
        cheap_min_score_fn: Fmin,
        cheap_max_score_fn: Fmax,
        customizations: Option<Customizations>,
    ) -> Self {
        Self {
            expensive_score_fn,
            cheap_min_score_fn,
            cheap_max_score_fn,
            customizations: customizations.unwrap_or(Customizations {
                color: DEFAULT_COLOR.into(),
                head: DEFAULT_HEAD.into(),
                tail: DEFAULT_TAIL.into(),
            }),
        }
    }

    fn solve(&self, game: &Game, deadline: &Instant, max_depth: usize) -> (Direction, S1) {
        let scores = scores::Scoretree::new(deadline.clone());

        solve::solve(
            game,
            &self.expensive_score_fn,
            &self.cheap_min_score_fn,
            &self.cheap_max_score_fn,
            scores.clone(),
            deadline.clone(),
            max_depth,
        );

        let move_scores = scores.get_scores(&vec![Vec::from(ALL_DIRECTIONS)]);
        let (mut top_move, mut top_score) = (Direction::Down, None);
        for dir in ALL_DIRECTIONS {
            if !certain_death(
                // This check is to compensate for "paranoid" scoring functions that
                // do not distiguish between maybe dieing, and certain instant death.
                // It's not perfect, but it makes the snake survive some
                // scenarios in which an enemy snake doesn't go in for the kill.
                &game,
                &game.you,
                &game.you.head.neighbour(dir),
                game.you.health,
            ) {
                if let Some(score) = move_scores.get(&vec![dir]) {
                    if top_score == None || top_score.unwrap() < score {
                        top_score = Some(score);
                        top_move = dir;
                    }
                } else {
                    println!("WARNING: did not get a top level score for {}", dir)
                }
            }
        }

        let top_score = match top_score {
            Some(s) => s.clone(),
            None => {
                println!("WARNING: did not get a single valid score, returning score for current board instead");
                (self.expensive_score_fn)(game)
            }
        };

        (top_move, top_score)
    }
}

impl<Fscore, Fmin, Fmax, S1, S2, S3> Battlesnake for Spaceheater2<Fscore, Fmin, Fmax, S1, S2, S3>
where
    Fscore: Fn(&Game) -> S1,
    Fmin: Fn(&Game) -> S2,
    Fmax: Fn(&Game) -> S3,
    S1: Ord + Display + Clone + Send,
    S2: Ord + PartialOrd<S1>,
    S3: Ord + PartialOrd<S1>,
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
        let (best_dir, top_score) = self.solve(&game, &deadline, usize::MAX);

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: format!("{}", top_score),
        })
    }
}
