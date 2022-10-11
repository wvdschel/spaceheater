use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    ops::{Deref, DerefMut},
    sync,
};

use serde::{Deserialize, Serialize};

use crate::protocol;

#[derive(Serialize, Deserialize)]
pub struct Game {
    start_request: protocol::Request,
    end_request: Option<protocol::Request>,
    moves: Vec<(protocol::Request, Option<protocol::MoveResponse>)>,
}

impl Game {
    fn new(start_request: protocol::Request) -> Self {
        Self {
            start_request,
            end_request: None,
            moves: Vec::new(),
        }
    }

    fn save(&self) -> Result<String, String> {
        let (snake_name, game_id) = game_id(&self.start_request);
        let filename = format!(
            "logs/{}.json",
            sanitize_filename::sanitize(format!("{}_{}", snake_name, game_id))
        );
        match serde_json::to_string(self) {
            Ok(json) => match File::create(&filename) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        Err(e.to_string())
                    } else {
                        Ok(filename)
                    }
                }

                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct GameLogger {
    open_games: sync::Mutex<HashMap<(String, String), Game>>,
}

impl GameLogger {
    pub fn new() -> Self {
        Self {
            open_games: sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn new_game(&mut self, start_request: &protocol::Request) {
        let mut open_games = self.open_games.lock().unwrap();
        open_games.insert(game_id(&start_request), Game::new(start_request.clone()));
    }

    pub fn end_game(&mut self, end_request: &protocol::Request) {
        let mut open_games = self.open_games.lock().unwrap();
        if let Some(mut game) = open_games.remove(&game_id(&end_request)) {
            game.end_request = Some(end_request.clone());
            match game.save() {
                Ok(filename) => println!("saved game replay in {}", filename),
                Err(e) => println!("warning: failed to record game: {}", e),
            }
        } else {
            println!(
                "end move for unknown game {} {}",
                end_request.you.name, end_request.game.id
            );
        }
    }

    pub fn log_move(
        &mut self,
        request: &protocol::Request,
        response: Option<&protocol::MoveResponse>,
    ) {
        let mut open_games = self.open_games.lock().unwrap();

        if let Some(game) = open_games.get_mut(&game_id(&request)) {
            game.moves
                .push((request.clone(), response.map(|r| r.clone())));
        } else {
            println!(
                "move for unknown game {} {}",
                request.you.name, request.game.id
            );
        }
    }
}

fn game_id(req: &protocol::Request) -> (String, String) {
    (req.you.name.clone(), req.game.id.clone())
}
