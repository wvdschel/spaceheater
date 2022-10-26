mod simple;
use std::collections::HashMap;

pub use simple::SimpleSnake;
pub mod spaceheater;
pub use spaceheater::SpaceHeater;

use crate::{logic, protocol::Customizations, Battlesnake};

pub fn snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(SimpleSnake {}));
    snakes.insert(
        "battlesnack".to_string(),
        Box::new(SpaceHeater::new(
            logic::scoring::tournament_voronoi,
            Customizations {
                color: "#533d6a".to_string(),
                head: "caffeine".to_string(),
                tail: "cosmic-horror".to_string(),
            },
        )),
    );

    snakes
}
