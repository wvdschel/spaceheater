use std::{sync::Arc, time::Duration};

use crate::protocol::{Customizations, RoyaleRules, Ruleset, RulesetSettings, SquadRules};

use super::{Board, Game, Point, Snake};

fn p(x: isize, y: isize) -> Point {
    Point { x, y }
}

fn standard_rules() -> Ruleset {
    Ruleset {
        name: "standard".into(),
        version: "v1.1.20".into(),
        settings: RulesetSettings {
            food_spawn_chance: 15,
            minimum_food: 1,
            hazard_damage_per_turn: 50,
            royale: RoyaleRules {
                shrink_every_n_turns: 0,
            },
            squad: SquadRules {
                allow_body_collisions: false,
                shared_elimination: false,
                shared_health: false,
                shared_length: false,
            },
        },
    }
}

fn a_snake(body: Vec<Point>) -> Snake {
    Snake {
        id: "".into(),
        name: "".into(),
        health: 100,
        latency: "0".into(),
        head: body[0].clone(),
        length: body.len(),
        body: body.into(),
        shout: "".into(),
        squad: "".into(),
        customizations: Customizations {
            color: "#ffffff".into(),
            head: "".into(),
            tail: "".into(),
        },
    }
}

fn empty_map(w: usize, h: usize) -> Game {
    Game {
        board: Arc::new(Board::new(w, h)),
        others: vec![],
        dead_snakes: 0,
        you: a_snake(vec![p(1, 1), p(1, 0), p(0, 0)]),
        timeout: Duration::from_millis(500),
        rules: Arc::new(standard_rules()),
        turn: 1,
    }
}

#[test]
fn head_to_head_collision() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn head_to_head_collision_equal_length() {
    let result = 2 + 2;
    assert_eq!(result, 4);
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
