mod simple;
pub mod spaceheater3;

pub use simple::SimpleSnake;
pub use spaceheater3::Spaceheater3;
use std::{collections::HashMap, fs};

use crate::{logic::scoring, protocol::Customizations, Battlesnake};

const CONFIG_DIR: &str = "./cfg";

pub fn snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(SimpleSnake {}));
    snakes.insert(
        "spaceheater3".to_string(),
        Box::new(Spaceheater3::new(
            scoring::tournament_score,
            Some(Customizations {
                color: "#FF2400".to_string(),
                head: "workout".to_string(),
                tail: "rocket".to_string(),
            }),
        )),
    );
    snakes.insert(
        "spaceheater_winter".to_string(),
        Box::new(Spaceheater3::new(
            scoring::winter::Config::<{ u16::MAX }> {
                points_per_food: 30,
                points_per_tile: 10,
                points_per_length_rank: -20,
                points_per_health: 1,
                points_per_distance_to_food: -1,
                points_per_kill: 100,
                points_per_turn_survived: 300,
                points_per_distance_to_smaller_enemies: -1,
                points_when_dead: -1000000,
                hungry_mode_max_health: 35,
                hungry_mode_food_multiplier: 6.0,
                food_distance_cap: 20,
                enemy_distance_cap: 20,
            },
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );

    snakes
}
