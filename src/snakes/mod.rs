mod simple;
pub mod spaceheater;
pub mod spaceheater2;

pub use simple::SimpleSnake;
pub use spaceheater::SpaceHeater;
pub use spaceheater2::Spaceheater2;
use std::{collections::HashMap, sync::Arc};

use crate::{logic, protocol::Customizations, Battlesnake};

pub fn snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(SimpleSnake {}));
    snakes.insert(
        "spaceheater".to_string(),
        Box::new(SpaceHeater::new(
            logic::scoring::tournament_voronoi,
            Customizations {
                color: "#E77200".to_string(),
                head: "workout".to_string(),
                tail: "rocket".to_string(),
            },
        )),
    );
    snakes.insert(
        "spaceheater2".to_string(),
        Box::new(Spaceheater2::new(
            logic::scoring::tournament_voronoi,
            logic::scoring::tournament_voronoi,
            None,
        )),
    );

    snakes
}
