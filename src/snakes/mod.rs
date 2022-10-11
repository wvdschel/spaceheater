mod simple;
use std::collections::HashMap;

pub use simple::SimpleSnake;
mod solid;
pub use solid::SolidSnake;
mod spaceheater;
pub use spaceheater::SpaceHeater;

use crate::Battlesnake;

pub fn snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(SimpleSnake {}));
    snakes.insert("solid".to_string(), Box::new(SolidSnake {}));
    snakes.insert("spaceheater".to_string(), Box::new(SpaceHeater::new()));

    snakes
}
