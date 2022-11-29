mod simple;
pub mod spaceheater3;

pub use simple::SimpleSnake;
pub use spaceheater3::Spaceheater3;
use std::collections::HashMap;

use crate::{logic, protocol::Customizations, Battlesnake};

pub fn snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(SimpleSnake {}));
    snakes.insert(
        "spaceheater3".to_string(),
        Box::new(Spaceheater3::new(
            logic::scoring::tournament_score,
            Some(Customizations {
                color: "#DA2C43".to_string(),
                head: "smile".to_string(),
                tail: "nr-booster".to_string(),
            }),
            false,
        )),
    );
    snakes.insert(
        "spaceheater3_mc".to_string(),
        Box::new(Spaceheater3::new(
            logic::scoring::tournament_score,
            Some(Customizations {
                color: "#FF2400".to_string(),
                head: "workout".to_string(),
                tail: "rocket".to_string(),
            }),
            true,
        )),
    );

    snakes
}
