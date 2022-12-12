use serde::{Deserialize, Serialize};
use std::cmp;

use rand::Rng;

use crate::{
    logic::Game,
    snakes::Spaceheater3,
    util::gauntlet::{GeneticConfig, RandomConfig},
};

pub use self::floodfill::floodfill;

use super::Scorer;

mod floodfill;
pub mod floodfill_baseline;

pub type NumType = u16;
pub const NO_SNAKE: u8 = u8::MAX;
const MAX_SNAKES: usize = 12;

#[derive(Clone, Debug)]
pub struct SnakeScore {
    pub food_count: NumType,
    pub tile_count: NumType,
    pub hazard_count: NumType,
    pub food_distance: NumType,
    pub food_at_min_distance: NumType,
    pub distance_to_collision: [NumType; MAX_SNAKES],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Config<const MAX_DISTANCE: NumType> {
    pub points_per_food: i64,
    pub points_per_tile: i64,
    pub points_per_hazard: i64,
    pub points_per_length_rank: i64,
    pub points_per_health: i64,
    pub points_per_distance_to_food: i64,
    pub food_distance_cap: NumType,
    pub points_per_kill: i64,
    pub points_per_turn_survived: i64,
    // should be balanced with points_per_length_rank: being longer should outweigh
    // the penalties of being far away from a smaller snake
    pub points_per_distance_to_smaller_enemies: i64,
    pub enemy_distance_cap: NumType,
    pub points_when_dead: i64,
    pub hungry_mode_max_health: i8,
    pub hungry_mode_food_multiplier: f64,
}

impl<const MAX_DISTANCE: NumType> RandomConfig for Config<MAX_DISTANCE> {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            points_per_food: rng.gen_range(0..30),
            points_per_tile: rng.gen_range(0..30),
            points_per_hazard: rng.gen_range(-10..0),
            points_per_length_rank: rng.gen_range(-200..10),
            points_per_health: rng.gen_range(0..30),
            points_per_distance_to_food: rng.gen_range(-30..5),
            points_per_kill: rng.gen_range(0..1000),
            points_per_turn_survived: rng.gen_range(0..1000),
            points_per_distance_to_smaller_enemies: rng.gen_range(-30..5),
            points_when_dead: -10000000,
            hungry_mode_max_health: rng.gen_range(15..70),
            hungry_mode_food_multiplier: rng.gen_range((1.0)..(15.0)),
            food_distance_cap: rng.gen_range(3..50),
            enemy_distance_cap: rng.gen_range(3..50),
        }
    }
}

impl<const MAX_DISTANCE: NumType> GeneticConfig for Config<MAX_DISTANCE> {
    fn evolve(&self) -> Box<dyn GeneticConfig> {
        let mut rng = rand::thread_rng();

        let mut res = self.clone();
        let mul = rng.gen_range::<i64, _>(-3..3).pow(2);
        match rng.gen_range(0..13) {
            0 => res.points_per_food += mul,
            1 => res.points_per_tile += mul,
            2 => res.points_per_length_rank += 2 * mul,
            3 => res.points_per_health += mul,
            4 => res.points_per_distance_to_food += mul,
            5 => res.points_per_kill += 5 * mul,
            6 => res.points_per_turn_survived += 5 * mul,
            7 => res.points_per_distance_to_smaller_enemies += mul,
            8 => {
                res.hungry_mode_max_health =
                    cmp::min(100, cmp::max(0, res.hungry_mode_max_health + mul as i8))
            }
            9 => res.hungry_mode_food_multiplier += 0.05 * mul as f64,
            10 => res.food_distance_cap = cmp::max(1, res.food_distance_cap + mul as NumType),
            11 => res.enemy_distance_cap = cmp::max(1, res.enemy_distance_cap + mul as NumType),
            12 => res.points_per_hazard += mul,
            _ => unreachable!(),
        }

        Box::new(res)
    }

    fn battlesnake(&self) -> Box<dyn crate::Battlesnake + Sync + Send> {
        Box::new(Spaceheater3::new(self.clone(), None))
    }

    fn load(&mut self, cfg: &str) {
        *self = Self::try_from(cfg).unwrap();
    }

    fn boxed_clone(&self) -> Box<dyn GeneticConfig> {
        Box::new(self.clone())
    }
}

impl<const MAX_DISTANCE: NumType> ToString for Config<MAX_DISTANCE> {
    fn to_string(&self) -> String {
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();
        let stringified: Vec<String> = encoded.iter().map(|b| format!("{:02x}", b)).collect();
        stringified.join("")
    }
}

impl<const MAX_DISTANCE: NumType> TryFrom<&str> for Config<MAX_DISTANCE> {
    type Error = ();

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let mut bytes = Vec::<u8>::with_capacity(v.len() / 2);
        for i in (0..v.len()).step_by(2) {
            if let Ok(b) = u8::from_str_radix(&v[i..i + 2], 16) {
                bytes.push(b);
            } else {
                return Err(());
            }
        }
        match bincode::deserialize::<Self>(bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        }
    }
}

#[test]
fn hex_encoded_config() {
    let cfg = Config::<{ NumType::MAX }>::random();
    let cfg_str = cfg.to_string();

    println!("as string: {}", cfg_str);

    let cfg_parsed = Config::<{ NumType::MAX }>::try_from(cfg_str.as_str()).unwrap();

    assert_eq!(cfg, cfg_parsed);
}

impl<const MAX_DISTANCE: NumType> Scorer for Config<MAX_DISTANCE> {
    fn score(&self, game: &Game) -> i64 {
        let mut score: i64 = 0;
        score += self.points_per_kill * game.dead_snakes as i64;
        score += self.points_per_turn_survived * game.turn as i64;

        if game.you.dead() {
            score -= self.points_per_turn_survived + self.points_per_kill;
            score += self.points_when_dead;
            return score;
        }

        let flood_info = floodfill::<MAX_DISTANCE>(game);

        score += self.points_per_health * game.you.health as i64;
        score += self.points_per_tile * flood_info[0].tile_count as i64;
        score += self.points_per_hazard * flood_info[0].hazard_count as i64;

        let mut length_rank = 0;
        for (i, snake) in game.others.iter().enumerate() {
            if snake.length >= game.you.length {
                length_rank += 1;
            } else {
                score += self.points_per_distance_to_smaller_enemies
                    * cmp::min(
                        flood_info[0].distance_to_collision[i + 1],
                        self.enemy_distance_cap,
                    ) as i64;
            }
        }
        score += self.points_per_length_rank * length_rank;

        let mut food_score = self.points_per_food * flood_info[0].food_count as i64;
        food_score += self.points_per_distance_to_food
            * cmp::min(flood_info[0].food_distance, self.food_distance_cap) as i64;
        if game.you.health < self.hungry_mode_max_health {
            food_score = f64::round(self.hungry_mode_food_multiplier * food_score as f64) as i64;
        }
        score += food_score;

        score
    }
}
