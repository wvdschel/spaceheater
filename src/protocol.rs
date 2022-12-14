use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Default, Deserialize, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Point {
    pub x: i8,
    pub y: i8,
}

// See https://docs.battlesnake.com/api/requests/info
#[derive(Serialize, Deserialize)]
pub struct SnakeInfo {
    pub apiversion: String,
    pub author: String,
    pub color: String,
    pub head: String,
    pub tail: String,
    pub version: String,
}

// See https://docs.battlesnake.com/api/objects/game
#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub map: String,
    pub timeout: isize,
    pub source: String,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
    pub settings: RulesetSettings,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RulesetSettings {
    pub food_spawn_chance: usize,
    pub minimum_food: usize,
    pub hazard_damage_per_turn: isize,
    pub royale: RoyaleRules,
    pub squad: SquadRules,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoyaleRules {
    pub shrink_every_n_turns: usize,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SquadRules {
    pub allow_body_collisions: bool,
    pub shared_elimination: bool,
    pub shared_health: bool,
    pub shared_length: bool,
}

// See https://docs.battlesnake.com/api/objects/board
#[derive(Serialize, Deserialize, Clone)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<Point>,
    pub hazards: Vec<Point>,
    pub snakes: Vec<Snake>,
}

// See https://docs.battlesnake.com/api/objects/battlesnake
#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Snake {
    pub id: String,
    pub name: String,
    pub health: isize,
    pub body: VecDeque<Point>,
    pub latency: String,
    pub head: Point,
    pub length: usize,
    pub shout: String,
    pub squad: String,
    pub customizations: Customizations,
}

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Customizations {
    pub color: String,
    pub head: String,
    pub tail: String,
}

// Request body for game start, game end and move endpoints
#[derive(Serialize, Deserialize, Clone)]
pub struct Request {
    pub game: Game,
    pub turn: usize,
    pub board: Board,
    pub you: Snake,
}

// Response body for move endpoint
#[derive(Serialize, Deserialize, Clone)]
pub struct MoveResponse {
    #[serde(rename = "move")]
    pub direction: Direction,
    pub shout: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

pub const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];
