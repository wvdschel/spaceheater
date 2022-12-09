mod simple;
pub mod spaceheater3;

pub use simple::SimpleSnake;
pub use spaceheater3::Spaceheater3;
use std::collections::HashMap;

use crate::{logic::scoring, protocol::Customizations, Battlesnake};

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
            scoring::winter::Config::<{ u16::MAX }>::try_from("0d000000000000000c00000000000000b2ffffffffffffff180000000000000002000000000000000f005500000000000000d502000000000000fbffffffffffffff2800806967ffffffffff2a435013210df72640").unwrap(),
            Some(Customizations {
                color: "#03befc".to_string(),
                head: "scarf".to_string(),
                tail: "coffee".to_string(),
            }),
        )),
    );

    snakes
}
