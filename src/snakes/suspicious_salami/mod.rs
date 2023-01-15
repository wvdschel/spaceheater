use std::{
    cmp::Ordering,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use protocol::Direction;
use rand::Rng;

use crate::{
    log,
    logic::{self, Game},
    protocol::{self, Customizations, ALL_DIRECTIONS},
    Battlesnake,
};

use super::spaceheater3::util::{all_sensible_enemy_moves, certain_death};

pub const DEFAULT_COLOR: &str = "#FF5C75";
pub const DEFAULT_HEAD: &str = "safe";
pub const DEFAULT_TAIL: &str = "round-bum";
const MAX_DEPTH: usize = 1000;
const LATENCY_MARGIN: Duration = Duration::from_millis(100);

pub struct Salami<S>
where
    S: logic::scoring::Scorer + Sync + Clone,
{
    scorer: S,
    customizations: Customizations,
}

impl<S> Battlesnake for Salami<S>
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

        let (best_dir, top_score) = self.solve(game, &deadline);

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: format!("{}", top_score),
        })
    }
}

impl<S> Salami<S>
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
        }
    }

    pub fn solve(&self, game: Game, deadline: &Instant) -> (Direction, i64) {
        let (tx, rx) = channel();
        let scorer = self.scorer.clone();
        let deadline = deadline.clone();
        thread::spawn(move || {
            let mut game = game;
            game.turn = 0;
            let score = scorer.score(&game) as f64;
            let mut root = Max {
                game,
                children: vec![],
                score,
                visit_count: 1,
            };

            while Instant::now() < deadline {
                if root.visit(&scorer, &deadline, MAX_DEPTH).is_none() {
                    break;
                };
            }

            // Return the highest scoring child
            if root.children.len() == 0 {
                println!("root has no children, returning up");
                let _ = tx.send((Direction::Up, i64::MIN));
                return;
            }
            println!("MCTS: visited root node {} times", root.visit_count);
            for c in &root.children {
                println!("MCTS: {} -> {}", c.my_move, c.score);
            }
            root.children
                .sort_by(|c1, c2| c2.score.partial_cmp(&c1.score).unwrap_or(Ordering::Equal));
            let _ = tx.send((
                root.children[0].my_move,
                root.children[0].score.round() as i64,
            ));
        });

        rx.recv().unwrap()
    }
}

struct Min {
    children: Vec<Max>,
    score: f64,
    visit_count: i64,
    my_move: Direction,
}

impl Min {
    fn visit<S>(
        &mut self,
        game: &Game,
        scorer: &S,
        deadline: &Instant,
        max_depth: usize,
    ) -> Option<f64>
    where
        S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    {
        self.generate_children(game, scorer);

        let mut max_score = f64::NEG_INFINITY;
        let mut sum = 0f64;

        for child in &self.children {
            if max_score < child.score {
                max_score = child.score;
            }
            sum += child.score as f64;
        }

        let mut rng = rand::thread_rng();
        let random_value = rng.gen_range(0f64..1f64);
        let mut accumulated_odds = 0f64;
        let child_count = self.children.len();

        log!(
            "min turn {}: random value = {}, number of children = {}",
            game.turn,
            random_value,
            child_count
        );

        for c in &mut self.children {
            let chance = min_child_odds(c.score, child_count, max_score, sum);
            accumulated_odds += chance;
            if random_value <= accumulated_odds {
                log!(
                    "picking child with odds {} (score {} of {}), total odds now {}",
                    chance,
                    c.score,
                    sum,
                    accumulated_odds
                );
                // visit child
                let res = c.visit(scorer, deadline, max_depth - 1);
                // update own score
                if let Some(score) = res {
                    self.visit_count += 1;
                    let count = self.visit_count as f64;
                    let new_score = self.score * (count - 1.0) / count + score / count;
                    self.score = new_score;
                }
                // return score of visited end node
                return res;
            }
            log!(
                "not picking child with odds {} (score {} of {}), total odds now {}",
                chance,
                c.score,
                sum,
                accumulated_odds
            );
        }

        panic!(
            "turn {}: end of Min::visit should not be reached!",
            game.turn
        );
    }

    fn generate_children<S>(&mut self, game: &Game, scorer: &S)
    where
        S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    {
        if self.children.len() == 0 {
            for combo in all_sensible_enemy_moves(game) {
                let mut game = game.clone();
                game.execute_moves(self.my_move, &combo);
                let score = scorer.score(&game) as f64;
                self.children.push(Max {
                    game,
                    children: vec![],
                    score,
                    visit_count: 1,
                });
            }
        }
    }
}

struct Max {
    game: Game,
    children: Vec<Min>,
    score: f64,
    visit_count: i64,
}

impl Max {
    fn visit<S>(&mut self, scorer: &S, deadline: &Instant, max_depth: usize) -> Option<f64>
    where
        S: logic::scoring::Scorer + Send + Sync + Clone + 'static,
    {
        if Instant::now() > *deadline {
            return None;
        }
        if self.game.you.dead() || max_depth == 0 {
            log!(
                "bound reached at turn {} (dead={}), returning {}",
                self.game.turn,
                self.game.you.dead(),
                self.score
            );
            return Some(self.score);
        }

        self.generate_children();

        let mut min_score = f64::INFINITY;
        let mut sum = 0f64;

        for child in &self.children {
            if min_score > child.score {
                min_score = child.score;
            }
            sum += child.score as f64;
        }

        let mut rng = rand::thread_rng();
        let random_value = rng.gen_range(0f64..1f64);
        let mut accumulated_odds = 0f64;
        let child_count = self.children.len();

        log!(
            "max turn {}: random value = {}, number of children = {}",
            self.game.turn,
            random_value,
            child_count
        );

        for c in &mut self.children {
            let chance = max_child_odds(c.score, child_count, min_score, sum);
            accumulated_odds += chance;
            if random_value <= accumulated_odds {
                log!(
                    "picking child {} with odds {} (score {} of {}), total odds now {}",
                    c.my_move,
                    chance,
                    c.score,
                    sum,
                    accumulated_odds
                );

                // visit child
                let res = c.visit(&self.game, scorer, deadline, max_depth);
                // update own score
                if let Some(score) = res {
                    self.visit_count += 1;
                    let count = self.visit_count as f64;
                    let new_score =
                        self.score as f64 * (count - 1.0) / count + score as f64 / count;
                    self.score = new_score;
                }
                // return score of visited end node
                return res;
            }

            log!(
                "not picking child {} with odds {} (score {} of {}), total odds now {}",
                c.my_move,
                chance,
                c.score,
                sum,
                accumulated_odds
            );
        }

        panic!(
            "turn {}: end of Max::visit should not be reached!",
            self.game.turn
        );
    }

    fn generate_children(&mut self) {
        if self.children.len() == 0 {
            for my_dir in ALL_DIRECTIONS {
                let mut my_pos = self.game.you.head.neighbour(my_dir);
                self.game.warp(&mut my_pos);
                if !certain_death(&self.game, &self.game.you, &my_pos) {
                    self.children.push(Min {
                        children: vec![],
                        score: self.score,
                        visit_count: 1,
                        my_move: my_dir,
                    });
                }
            }
            if self.children.len() == 0 {
                // rejected all directions, but we still have to move because certain death.
                self.children.push(Min {
                    children: vec![],
                    score: self.score,
                    visit_count: 1,
                    my_move: Direction::Up,
                });
            }
        } else {
            self.children.sort_unstable_by(|c1, c2| {
                c2.score.partial_cmp(&c1.score).unwrap_or(Ordering::Equal)
            });
        }
    }
}

fn min_child_odds(score: f64, count: usize, max_score: f64, sum: f64) -> f64 {
    // Convert all scores to strictly positive values
    let score = max_score as f64 - score as f64;
    let avg_score = max_score as f64 - (sum / count as f64);
    let sum = avg_score * count as f64;

    let mut odds = 1f64 / count as f64;
    if sum > 0.1f64 {
        let total_fraction = score / sum;

        // don't use the fraction of the score as the only factor, or minimal scores will never be revisited
        odds = total_fraction * 0.9 + odds * 0.1;
    }

    odds
}

fn max_child_odds(score: f64, count: usize, min_score: f64, sum: f64) -> f64 {
    let correction = if min_score < 0f64 {
        min_score.abs() as f64
    } else {
        0f64
    };

    let sum = sum + (count as f64 * correction);
    let score = score as f64 + correction;

    let mut odds = 1f64 / count as f64;
    if sum > 0.1f64 {
        let total_fraction = score / sum;

        // don't use the fraction of the score as the only factor, or minimal scores will never be revisited
        odds = total_fraction * 0.98 + odds * 0.02;
    }

    odds
}
