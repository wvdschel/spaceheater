use std::io::prelude::*;
use std::{collections::HashMap, fs::File, sync};

use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::{protocol, Battlesnake};

#[derive(Serialize, Deserialize)]
pub struct Game {
    start_request: protocol::Request,
    end_request: Option<protocol::Request>,
    moves: Vec<(protocol::Request, Option<protocol::MoveResponse>)>,
}

type Result = std::result::Result<String, String>;

fn into_result<T, E: ToString>(v: std::result::Result<T, E>) -> std::result::Result<T, String> {
    match v {
        Ok(t) => Ok(t),
        Err(e) => Err(e.to_string()),
    }
}

impl Game {
    fn new(start_request: protocol::Request) -> Self {
        Self {
            start_request,
            end_request: None,
            moves: Vec::new(),
        }
    }

    pub fn load(source: &mut dyn std::io::Read) -> std::result::Result<Game, String> {
        let mut buf = Vec::<u8>::new();
        into_result(source.read_to_end(&mut buf))?;

        let mut d = GzDecoder::new(buf.as_ref());
        let mut decomp_buf = Vec::new();
        if let Ok(_) = d.read_to_end(&mut decomp_buf) {
            buf = decomp_buf;
        }

        let game = into_result(serde_json::from_slice(&buf))?;
        Ok(game)
    }

    pub fn replay(&self, snake: &dyn Battlesnake) {
        _ = snake.start(&self.start_request);
        for (r, _) in &self.moves {
            _ = snake.make_move(r);
        }
        if let Some(end_request) = &self.end_request {
            _ = snake.end(end_request);
        }
    }

    fn save(&self) -> Result {
        let (snake_name, game_id) = game_id(&self.start_request);
        let filename = format!(
            "logs/{}.json.gz",
            sanitize_filename::sanitize(format!("{}_{}", snake_name, game_id))
        );
        let json = into_result(serde_json::to_string(self))?;
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        into_result(e.write_all(json.as_bytes()))?;
        let compressed_bytes = into_result(e.finish())?;

        let mut file = into_result(File::create(&filename))?;
        if let Err(e) = file.write_all(&compressed_bytes) {
            Err(e.to_string())
        } else {
            Ok(filename)
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
