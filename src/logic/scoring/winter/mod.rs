use serde::{Deserialize, Serialize};
use std::cmp;

use rand::Rng;

use crate::{
    logic::Game,
    snakes::Spaceheater3,
    util::gauntlet::{GeneticConfig, RandomConfig},
};

const ENCODING: &str = include_str!("encoding.txt");

pub use self::floodfill::floodfill;

use super::Scorer;

mod floodfill;

pub type NumType = u8;
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
    pub points_per_food: i32,
    pub points_per_tile: i32,
    pub points_per_hazard: i32,
    pub points_per_length_rank: i32,
    pub points_per_health: i32,
    pub points_per_distance_to_food: i32,
    pub food_distance_cap: u16,
    pub points_per_kill: i32,
    pub points_per_turn_survived: i32,
    // should be balanced with points_per_length_rank: being longer should outweigh
    // the penalties of being far away from a smaller snake
    pub points_per_distance_to_smaller_enemies: i32,
    pub enemy_distance_cap: u16,
    pub points_when_dead: i32,
    pub hungry_mode_max_health: i8,
    pub hungry_mode_food_multiplier: f32,
    pub points_per_length_diff: i32,
    pub length_diff_cap: u8,
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
            points_per_kill: rng.gen_range(0..5000),
            points_per_turn_survived: rng.gen_range(0..1000),
            points_per_distance_to_smaller_enemies: rng.gen_range(-30..5),
            points_when_dead: -10000000,
            hungry_mode_max_health: rng.gen_range(15..70),
            hungry_mode_food_multiplier: rng.gen_range((1.0)..(15.0)),
            food_distance_cap: rng.gen_range(3..50),
            enemy_distance_cap: rng.gen_range(3..50),
            points_per_length_diff: rng.gen_range(0..150),
            length_diff_cap: rng.gen_range(0..10),
        }
    }
}

impl<const MAX_DISTANCE: NumType> GeneticConfig for Config<MAX_DISTANCE> {
    fn mutate(&self) -> Box<dyn GeneticConfig> {
        let mut rng = rand::thread_rng();

        let mut res = self.clone();
        let mul = rng.gen_range::<i32, _>(-5..5).pow(2);
        match rng.gen_range(0..15) {
            0 => res.points_per_food = mul,
            1 => res.points_per_tile += mul,
            2 => res.points_per_length_rank += 3 * mul,
            3 => res.points_per_health += mul,
            4 => res.points_per_distance_to_food += mul,
            5 => res.points_per_kill += 25 * mul,
            6 => res.points_per_turn_survived += 8 * mul,
            7 => res.points_per_distance_to_smaller_enemies += mul,
            8 => {
                res.hungry_mode_max_health =
                    cmp::min(100, cmp::max(0, res.hungry_mode_max_health + mul as i8))
            }
            9 => res.hungry_mode_food_multiplier += 0.05 * mul as f32,
            10 => res.food_distance_cap = cmp::max(1, res.food_distance_cap + mul as u16),
            11 => res.enemy_distance_cap = cmp::max(1, res.enemy_distance_cap + mul as u16),
            12 => res.points_per_hazard += mul,
            13 => res.points_per_length_diff += mul * 3,
            14 => res.length_diff_cap += mul as u8,
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

    fn try_crossover(&self, other_genes: &str, ratio_other: f64) -> Option<Box<dyn GeneticConfig>> {
        let other = if let Ok(v) = Self::try_from(other_genes) {
            v
        } else {
            return None;
        };
        let mut rng = rand::thread_rng();

        let res = Self {
            points_per_food: if rng.gen_bool(ratio_other) {
                other.points_per_food
            } else {
                self.points_per_food
            },
            points_per_tile: if rng.gen_bool(ratio_other) {
                other.points_per_tile
            } else {
                self.points_per_tile
            },
            points_per_hazard: if rng.gen_bool(ratio_other) {
                other.points_per_hazard
            } else {
                self.points_per_hazard
            },
            points_per_length_rank: if rng.gen_bool(ratio_other) {
                other.points_per_length_rank
            } else {
                self.points_per_length_rank
            },
            points_per_health: if rng.gen_bool(ratio_other) {
                other.points_per_health
            } else {
                self.points_per_health
            },
            points_per_distance_to_food: if rng.gen_bool(ratio_other) {
                other.points_per_distance_to_food
            } else {
                self.points_per_distance_to_food
            },
            food_distance_cap: if rng.gen_bool(ratio_other) {
                other.food_distance_cap
            } else {
                self.food_distance_cap
            },
            points_per_kill: if rng.gen_bool(ratio_other) {
                other.points_per_kill
            } else {
                self.points_per_kill
            },
            points_per_turn_survived: if rng.gen_bool(ratio_other) {
                other.points_per_turn_survived
            } else {
                self.points_per_turn_survived
            },
            points_per_distance_to_smaller_enemies: if rng.gen_bool(ratio_other) {
                other.points_per_distance_to_smaller_enemies
            } else {
                self.points_per_distance_to_smaller_enemies
            },
            enemy_distance_cap: if rng.gen_bool(ratio_other) {
                other.enemy_distance_cap
            } else {
                self.enemy_distance_cap
            },
            points_when_dead: if rng.gen_bool(ratio_other) {
                other.points_when_dead
            } else {
                self.points_when_dead
            },
            hungry_mode_max_health: if rng.gen_bool(ratio_other) {
                other.hungry_mode_max_health
            } else {
                self.hungry_mode_max_health
            },
            hungry_mode_food_multiplier: if rng.gen_bool(ratio_other) {
                other.hungry_mode_food_multiplier
            } else {
                self.hungry_mode_food_multiplier
            },
            points_per_length_diff: if rng.gen_bool(ratio_other) {
                other.points_per_length_diff
            } else {
                self.points_per_length_diff
            },
            length_diff_cap: if rng.gen_bool(ratio_other) {
                other.length_diff_cap
            } else {
                self.length_diff_cap
            },
        };

        Some(Box::new(res))
    }
}

impl<const MAX_DISTANCE: NumType> ToString for Config<MAX_DISTANCE> {
    fn to_string(&self) -> String {
        let characters: Vec<String> = ENCODING
            .split("")
            .filter(|v| v.len() != 0)
            .map(|c| c.to_string())
            .collect();
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();
        let stringified: Vec<String> = encoded
            .iter()
            .map(|b| characters[*b as usize].clone())
            .collect();
        stringified.join("")
    }
}

impl<const MAX_DISTANCE: NumType> TryFrom<&str> for Config<MAX_DISTANCE> {
    type Error = String;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let mut bytes = Vec::<u8>::with_capacity(v.len());
        let characters: Vec<char> = ENCODING
            .split("")
            .filter(|v| v.len() != 0)
            .map(|c| c.chars().next().unwrap_or('?'))
            .collect();

        for c in v.chars() {
            let pos = characters
                .iter()
                .enumerate()
                .find(|(_, &v)| v == c)
                .map(|(idx, _)| idx);
            match pos {
                Some(idx) => bytes.push(idx as u8),
                None => return Err(format!("no such character in encoding table: '{}'", c)),
            }
        }

        match bincode::deserialize::<Self>(bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("failed to deserialize config: {}", e)),
        }
    }
}

#[test]
fn string_encoded_config() {
    assert_eq!(ENCODING.chars().count(), 256);

    let cfg = Config::<{ NumType::MAX }>::random();

    let cfg_str = cfg.to_string();
    println!("as string: \"{}\"", cfg_str);

    let cfg_parsed = Config::<{ NumType::MAX }>::try_from(cfg_str.as_str()).unwrap();

    assert_eq!(cfg, cfg_parsed);
}

impl<const MAX_DISTANCE: NumType> Scorer for Config<MAX_DISTANCE> {
    fn score(&self, game: &Game) -> i64 {
        let mut score: i64 = 0;
        score += self.points_per_kill as i64 * game.dead_snakes as i64;
        score += self.points_per_turn_survived as i64 * game.turn as i64;

        if game.you.dead() {
            score -= self.points_per_turn_survived as i64 + self.points_per_kill as i64;
            score += self.points_when_dead as i64;
            return score;
        }

        let flood_info = floodfill::<MAX_DISTANCE>(game);

        score += self.points_per_health as i64 * game.you.health as i64;
        score += self.points_per_tile as i64 * flood_info[0].tile_count as i64;
        score += self.points_per_hazard as i64 * flood_info[0].hazard_count as i64;

        let mut length_rank = 0;
        for (i, snake) in game.others.iter().enumerate() {
            score += self.points_per_length_diff as i64
                * cmp::min(
                    self.length_diff_cap as i64,
                    cmp::max(
                        -(self.length_diff_cap as i64),
                        game.you.length as i64 - snake.length as i64,
                    ),
                ) as i64;
            if snake.length >= game.you.length {
                length_rank += 1;
            } else {
                score += self.points_per_distance_to_smaller_enemies as i64
                    * cmp::min(
                        flood_info[0].distance_to_collision[i + 1],
                        self.enemy_distance_cap as NumType,
                    ) as i64;
            }
        }
        score += self.points_per_length_rank as i64 * length_rank as i64;

        let mut food_score = self.points_per_food as i64 * flood_info[0].food_count as i64;
        food_score += self.points_per_distance_to_food as i64
            * cmp::min(
                flood_info[0].food_distance,
                self.food_distance_cap as NumType,
            ) as i64;
        if game.you.health < self.hungry_mode_max_health {
            food_score = f32::round(self.hungry_mode_food_multiplier * food_score as f32) as i64;
        }
        score += food_score;

        score
    }
}
