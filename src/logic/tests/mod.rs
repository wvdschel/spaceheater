use protocol::Point;

use crate::protocol::{self};

use super::{Direction, Game};

mod board;
mod floodfill;
mod snail_mode;

#[test]
fn head_to_head_collision_equal_length() {
    let request: protocol::Request =
        serde_json::from_str(include_str!("data/head_to_head_equal_length.json")).unwrap();
    let mut game = Game::from(&request);

    game.execute_moves(Direction::Down, &vec![Direction::Up, Direction::Up]);
    assert_eq!(game.you.health, 0);
    assert_eq!(game.others[0].health, 86);
    assert_eq!(game.dead_snakes, 2); // Our snake and 1 opponent have died
}

#[test]
fn food_pickup() {
    let request: protocol::Request =
        serde_json::from_str(include_str!("data/you_eat_food.json")).unwrap();
    let mut game = Game::from(&request);

    game.execute_moves(Direction::Right, &vec![]);
    assert_eq!(game.you.health, 100);
    assert_eq!(game.you.length, 4);
    assert_eq!(
        game.you.body,
        vec![
            Point { x: 5, y: 5 },
            Point { x: 4, y: 5 },
            Point { x: 3, y: 5 },
            Point { x: 3, y: 5 }
        ]
    );

    game.execute_moves(Direction::Right, &vec![]);
    assert_eq!(game.you.health, 99);
    assert_eq!(game.you.length, 4);
    assert_eq!(
        game.you.body,
        vec![
            Point { x: 6, y: 5 },
            Point { x: 5, y: 5 },
            Point { x: 4, y: 5 },
            Point { x: 3, y: 5 }
        ]
    );
}

#[test]
fn self_collision_when_wrapping() {
    let request: protocol::Request =
        serde_json::from_str(include_str!("data/self_collision_wrapped.json")).unwrap();
    let mut game = Game::from(&request);

    game.execute_moves(Direction::Up, &vec![]);
    assert_eq!(game.you.health, 0);
}
