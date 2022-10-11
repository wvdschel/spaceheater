pub mod gamelogger;
pub mod logic;
pub mod protocol;
pub mod snakes;
pub mod util;

pub trait Battlesnake {
    fn snake_info(&self) -> protocol::SnakeInfo;
    fn start(&self, req: &protocol::Request) -> Result<(), String>;
    fn end(&self, req: &protocol::Request) -> Result<(), String>;
    fn make_move(&self, req: &protocol::Request) -> Result<protocol::MoveResponse, String>;
}
