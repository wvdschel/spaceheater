use crate::logic::{Direction, Game};
use crate::protocol::ALL_DIRECTIONS;
use crate::{protocol, Battlesnake};

#[derive(Copy, Clone)]
pub struct SolidSnake {}

struct Score {
    confidence: f64, // Confidence of the evaluation function in the score, will be used as a weight for combining multiple evaulators
    score: f64,      // Negative scores bad, positive scores bad
}

trait GameEvaluator {
    fn score(&self, g: &Game) -> Score;
}

impl Battlesnake for SolidSnake {
    fn snake_info(&self) -> protocol::SnakeInfo {
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "General Error".to_string(),
            color: "#000000".to_string(),
            head: "missile".to_string(),
            tail: "missile".to_string(),
            version: "110ab".to_string(),
        }
    }

    fn start(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: &protocol::Request) -> Result<protocol::MoveResponse, String> {
        let game: Game = req.into();

        //let (tx, rx) = mpsc::channel();

        // Sleep for the time per move, leaving 50ms for latency
        std::thread::sleep(game.timeout - std::time::Duration::from_millis(50));

        Ok(protocol::MoveResponse {
            direction: Direction::Left,
            shout: "".to_string(),
        })
    }
}

impl SolidSnake {
    fn evaluators() -> Vec<(Box<dyn GameEvaluator>, f64)> {
        vec![(Box::new(Survival {}), 1.0)]
    }

    fn run_evaluator(&self, eval: &dyn GameEvaluator, game: &Game) {
        for my_dir in ALL_DIRECTIONS {
            let mut opponent_moves: Vec<Vec<Direction>> = vec![];
            for _ in game.others.iter() {
                if opponent_moves.is_empty() {
                    opponent_moves = vec![
                        vec![Direction::Up],
                        vec![Direction::Down],
                        vec![Direction::Left],
                        vec![Direction::Right],
                    ]
                } else {
                    let mut new_moves = Vec::with_capacity(opponent_moves.len() * 4);
                    for m in opponent_moves.iter() {
                        for omove in ALL_DIRECTIONS {
                            let mut m = m.clone();
                            m.push(omove);
                            new_moves.push(m);
                        }
                    }
                    opponent_moves = new_moves;
                }
            }
            let opponent_moves = opponent_moves;
        }
    }
}

struct Survival {}

impl GameEvaluator for Survival {
    fn score(&self, g: &Game) -> Score {
        let kill_bonus =
            g.dead_snakes as f64 / (1.0 + g.dead_snakes as f64 + g.others.len() as f64);
        if !g.contains_snake(&g.you.name) {
            return Score {
                score: -1.0f64 + kill_bonus,
                confidence: 1.0,
            };
        }
        Score {
            score: kill_bonus,
            confidence: 1.0,
        }
    }
}
