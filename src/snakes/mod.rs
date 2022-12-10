mod simple;
pub mod spaceheater3;

pub use simple::SimpleSnake;
pub use spaceheater3::Spaceheater3;
use std::collections::HashMap;

use crate::{logic::scoring, protocol::Customizations, Battlesnake};

const WINTER_CHAMPION: &str = "08000000000000000700000000000000fbffffffffffffff58ffffffffffffff0700000000000000f4ffffffffffffff0b004b010000000000000701000000000000fdffffffffffffff2000806967ffffffffff216dc437a494131440";

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
            scoring::winter::Config::<{ u16::MAX }>::try_from(WINTER_CHAMPION).unwrap(),
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );

    println!(
        "Winter champion config: {:?}",
        scoring::winter::Config::<{ u16::MAX }>::try_from(WINTER_CHAMPION).unwrap()
    );

    snakes
}
