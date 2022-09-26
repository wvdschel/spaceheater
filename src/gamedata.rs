use serde::{Deserialize, Serialize, Serializer, Deserializer};

#[derive(Serialize, Deserialize)]
pub struct Point {
    x: u64,
    y: u64,
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
    pub timeout: i64,
    pub source: String,
}

#[derive(Serialize, Deserialize)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
}

// See https://docs.battlesnake.com/api/objects/board
#[derive(Serialize, Deserialize)]
pub struct Board {
    pub height: u64,
    pub width: u64,
    pub food: Vec<Point>,
    pub hazards: Vec<Point>,
    pub snakes: Vec<Snake>,
}

// See https://docs.battlesnake.com/api/objects/battlesnake
#[derive(Serialize, Deserialize)]
pub struct Snake {
    id: String,
    name: String,
    health: u64,
    body: Vec<Point>,
    latency: u64,
    head: Point,
    length: u64,
    shout: String,
    squad: u64,
    customizations: Customizations,
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
    pub turn: u64,
    pub board: Board,
    pub you: Snake,
}

// Response body for move endpoint
#[derive(Serialize, Deserialize)]
pub struct MoveResponse {
    pub direction: Direction, #[serde(rename = "move")]
    pub shout: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up, Down, Left, Right
}