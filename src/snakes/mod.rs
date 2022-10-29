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
        "spaceheater_vt".to_string(),
        Box::new(SpaceHeater::new(logic::scoring::tournament_voronoi)),
    );

    snakes
}
