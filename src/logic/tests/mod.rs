use crate::protocol::{self};

use super::{Direction, Game};

mod board;

#[test]
fn head_to_head_collision() {}

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
fn head_to_body_collision() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn starvation() {}

#[test]
fn food_pickup() {}

#[test]
fn food_pickup_after_head_to_head() {}

#[test]
fn hazard_damage() {}

#[test]
fn stacked_hazard_damage() {}
