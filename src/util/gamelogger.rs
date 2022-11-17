use std::cmp;
use std::io::prelude::*;
use std::time::SystemTime;
use std::{collections::HashMap, fs::File, sync};

use flate2::bufread::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::{protocol, Battlesnake};

#[derive(Serialize, Deserialize)]
pub struct Game {
    pub timestamp: Option<String>,
    pub start_request: protocol::Request,
    pub end_request: Option<protocol::Request>,
    pub moves: Vec<(protocol::Request, Option<protocol::MoveResponse>)>,
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
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            timestamp: Some(format!("{}", start_time)),
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

    pub fn replay(
        &self,
        snake: &dyn Battlesnake,
        start_turn: Option<usize>,
        end_turn: Option<usize>,
        time_per_turn: Option<usize>,
    ) {
        let mut start_req = self.start_request.clone();
        if let Some(millis) = time_per_turn {
            start_req.game.timeout = millis as isize;
        }
        _ = snake.start(&start_req);
        let mut start = 0;
        let mut end = self.moves.len();
        if let Some(s) = start_turn {
            start = s;
        }
        if let Some(e) = end_turn {
            end = cmp::min(e + 1, end);
        }
        println!(
            "Replaying {} from turn {} until turn {} (game has {} turns)",
            self.start_request.game.id,
            start,
            end,
            self.moves.len()
        );

        if start < end {
            for (r, _) in &self.moves[start..end] {
                let mut req = r.clone();
                if let Some(millis) = time_per_turn {
                    req.game.timeout = millis as isize;
                }
                _ = snake.make_move(&req);
            }
        }
        if let Some(end_request) = &self.end_request {
            let mut end_req = end_request.clone();
            if let Some(millis) = time_per_turn {
                end_req.game.timeout = millis as isize;
            }
            _ = snake.end(&end_req);
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
