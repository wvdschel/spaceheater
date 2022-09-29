use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    pub fn neighbour(&self, d: Direction) -> Point {
        match d {
            Direction::Up => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }

    pub fn neighbours(&self) -> [(Direction, Point); 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .map(|d| (d, self.neighbour(d)))
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.x, self.y))
    }
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
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
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub map: String,
    pub timeout: isize,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
    pub settings: RulesetSettings,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RulesetSettings {
    pub food_spawn_chance: usize,
    pub minimum_food: usize,
    pub hazard_damage_per_turn: isize,
    pub royale: RoyaleRules,
    pub squad: SquadRules,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoyaleRules {
    pub shrink_every_n_turns: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadRules {
    pub allow_body_collisions: bool,
    pub shared_elimination: bool,
    pub shared_health: bool,
    pub shared_length: bool,
}

// See https://docs.battlesnake.com/api/objects/board
#[derive(Serialize, Deserialize)]
pub struct Board {
    pub height: usize,
    pub width: usize,
    pub food: Vec<Point>,
    pub hazards: Vec<Point>,
    pub snakes: Vec<Snake>,
}

// See https://docs.battlesnake.com/api/objects/battlesnake
#[derive(Serialize, Deserialize)]
pub struct Snake {
    pub id: String,
    pub name: String,
    pub health: usize,
    pub body: Vec<Point>,
    pub latency: String,
    pub head: Point,
    pub length: usize,
    pub shout: String,
    pub squad: String,
    pub customizations: Customizations,
}

#[derive(Serialize, Deserialize)]
pub struct Customizations {
    pub color: String,
    pub head: String,
    pub tail: String,
}

// Request body for game start, game end and move endpoints
#[derive(Serialize, Deserialize)]
pub struct Request {
    pub game: Game,
    pub turn: usize,
    pub board: Board,
    pub you: Snake,
}

// Response body for move endpoint
#[derive(Serialize, Deserialize)]
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

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl Display for Direction{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}