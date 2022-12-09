mod simple;
pub mod spaceheater3;

pub use simple::SimpleSnake;
pub use spaceheater3::Spaceheater3;
use std::collections::HashMap;

use crate::{logic::scoring, protocol::Customizations, Battlesnake};

const WINTER_CHAMPION: &str = "1c000000000000001200000000000000e3ffffffffffffff0a00000000000000ecffffffffffffff1700f802000000000000230200000000000002000000000000001d00806967ffffffffff323880406bf5561a40";

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
                points_per_food: 28,
                points_per_tile: 18,
                points_per_hazard: -10,
                points_per_length_rank: -29,
                points_per_health: 10,
                points_per_distance_to_food: -20,
                food_distance_cap: 23,
                points_per_kill: 760,
                points_per_turn_survived: 547,
                points_per_distance_to_smaller_enemies: 2,
                enemy_distance_cap: 29,
                points_when_dead: -10000000,
                hungry_mode_max_health: 50,
                hungry_mode_food_multiplier: 6.58492057400877,
            },
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );

    // println!(
    //     "Winter champion config: {:?}",
    //     scoring::winter::Config::<{ u16::MAX }>::try_from(WINTER_CHAMPION).unwrap()
    // );

    snakes
}
