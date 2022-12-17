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

const WINTER_CHAMPION: &str = "⠑⠀⠀⠀⠑⠀⠀⠀⣶⣿⣿⣿⢄⣿⣿⣿⠇⠀⠀⠀⣣⣿⣿⣿⠅⠀⡅⠁⠀⠀⡭⠃⠀⠀⣻⣿⣿⣿⠢⠀⢀⡩⡧⣿⠒⢦⠸⡚⡁⠱⠀⠀⠀⠃";

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
    println!(
        "Winter champion config: {:?} '{}'",
        champion_cfg,
        champion_cfg.to_string()
    );

    snakes.insert(
        "spaceheater_winter".to_string(),
        Box::new(Spaceheater3::new(
            champion_cfg,
            Some(Customizations {
                color: "#ff8400".to_string(),
                head: "workout".to_string(),
                tail: "rocket".to_string(),
            }),
        )),
    );
    snakes
}
