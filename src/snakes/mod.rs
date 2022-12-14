mod simple;
pub mod spaceheater3;

pub use simple::SimpleSnake;
pub use spaceheater3::Spaceheater3;
use std::collections::HashMap;

use crate::{
    logic::scoring::{self, winter},
    protocol::Customizations,
    Battlesnake,
};

const WINTER_CHAMPION: &str = "1100000011000000f6ffffff84ffffff07000000e3ffffff0500450100006d030000fbffffff2200806967ff12a6385a410a00000003";

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

    let champion_cfg =
        scoring::winter::Config::<{ winter::NumType::MAX }>::try_from(WINTER_CHAMPION).unwrap();

    let mut bigger_kill_value = champion_cfg.clone();
    bigger_kill_value.points_per_kill = bigger_kill_value.points_per_kill * 2 / 3;

    let mut no_enemy_distance_penalty = champion_cfg.clone();
    no_enemy_distance_penalty.points_per_distance_to_smaller_enemies = 0;

    let mut why_not_both = champion_cfg.clone();
    why_not_both.points_per_distance_to_smaller_enemies = 0;
    why_not_both.points_per_kill = why_not_both.points_per_kill * 2 / 3;

    let mut no_len = champion_cfg.clone();
    no_len.length_diff_cap = 0;
    no_len.points_per_length_diff = 0;

    snakes.insert(
        "spaceheater_bigger_kill".to_string(),
        Box::new(Spaceheater3::new(
            bigger_kill_value,
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );
    snakes.insert(
        "spaceheater_winter".to_string(),
        Box::new(Spaceheater3::new(
            champion_cfg,
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );
    snakes.insert(
        "spaceheater_no_enemy_distance".to_string(),
        Box::new(Spaceheater3::new(
            no_enemy_distance_penalty,
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );
    snakes.insert(
        "spaceheater_both".to_string(),
        Box::new(Spaceheater3::new(
            why_not_both,
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );
    snakes.insert(
        "spaceheater_no_len".to_string(),
        Box::new(Spaceheater3::new(
            no_len,
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );

    // println!(
    //     "Winter champion config: {:?}",
    //     scoring::winter::Config::<{ winter::NumType::MAX }>::try_from(WINTER_CHAMPION).unwrap()
    // );

    snakes
}
