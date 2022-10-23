mod simple;
use std::collections::HashMap;

pub use simple::SimpleSnake;
mod spaceheater;
pub use spaceheater::SpaceHeater;

use crate::{logic, Battlesnake};

pub fn snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(SimpleSnake {}));
    snakes.insert(
        "spaceheater".to_string(),
        Box::new(SpaceHeater::new(logic::scoring::survival_kills_length)),
    );
    snakes.insert(
        "spaceheater_v".to_string(),
        Box::new(SpaceHeater::new(logic::scoring::voronoi)),
    );

    snakes
}
