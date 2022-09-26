use crate::gamedata;

#[derive(Copy, Clone)]
pub struct SimpleSnake {}

impl crate::Battlesnake for SimpleSnake {
    fn snake_info(&self) -> gamedata::SnakeInfo {
        gamedata::SnakeInfo {
            apiversion: "1".to_string(),
            author: "General Error".to_string(),
            color: "#ffff00".to_string(),
            head: "silly".to_string(),
            tail: "sharp".to_string(),
            version: "106b".to_string(),
        }
    }

    fn start(&self, _: gamedata::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: gamedata::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: gamedata::Request) -> Result<gamedata::MoveResponse, String> {
        Ok(gamedata::MoveResponse {
            direction: gamedata::Direction::Down,
            shout: "".to_string(),
        })
    }
}