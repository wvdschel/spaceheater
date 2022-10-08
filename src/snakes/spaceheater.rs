use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    logic::{self, search, BoardLike, BoardOverlay, Direction, Game, Point, Tile},
    protocol::{self, ALL_DIRECTIONS},
    Battlesnake,
};

const MAX_LATENCY: Duration = Duration::from_millis(60);

fn look_ahead(game: &Game, deadline: &Instant, fork: usize) -> [(Direction, usize); 4] {
    if fork > 0 {
        fork_lookahead(game, deadline, fork - 1)
    } else {
        simple_lookahead(game, deadline)
    }
}

fn fork_lookahead(game: &Game, deadline: &Instant, fork: usize) -> [(Direction, usize); 4] {
    // TODO fork 'n shit
    simple_lookahead(game, deadline)
}

fn simple_lookahead(game: &Game, deadline: &Instant) -> [(Direction, usize); 4] {
    if &Instant::now() > deadline {
        return ALL_DIRECTIONS.map(|d| (d, 0));
    }
    let mut scores = HashMap::<Direction, usize>::from([
        (Direction::Up, 0),
        (Direction::Down, 0),
        (Direction::Right, 0),
        (Direction::Left, 0),
    ]);
    let start_turn = game.turn;
    let mut queue = VecDeque::new();
    for d in ALL_DIRECTIONS {
        let mut ng = game.clone();
        ng.execute_moves(d, vec![]);
        if ng.you.health > 0 {
            scores.insert(d, 1);
            queue.push_back((d, ng));
        }
    }

    println!("======== TURN {} ========", game.turn);
    println!("hp = {}", game.you.health);
    println!("{}", game.board);

    while &Instant::now() < deadline {
        if let Some((first_dir, game)) = queue.pop_front() {
            for dir in ALL_DIRECTIONS {
                let mut ng = game.clone();
                let score = ng.turn - start_turn;
                ng.execute_moves(dir, vec![]);
                if score == 1 {
                    println!("----- BEFORE -----");
                    println!("hp = {}", game.you.health);
                    println!("{}", game.board);

                    println!("----- TURN {}: GO {} -----", ng.turn, dir);
                    println!("hp = {}", ng.you.health);
                    println!("{}", ng.board);
                }
                if ng.you.health > 0 {
                    // Score is guarantueed to be >= scores[first_dir]
                    scores.insert(first_dir, score);
                    queue.push_back((first_dir, ng))
                }
            }
        } else {
            println!("reached end of game tree");
            break;
        }
    }

    ALL_DIRECTIONS.map(|d| (d, *scores.get(&d).unwrap()))
}

#[derive(Copy, Clone)]
pub struct SpaceHeater {}

impl Battlesnake for SpaceHeater {
    fn snake_info(&self) -> protocol::SnakeInfo {
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "General Error".to_string(),
            color: "#4A0E3D".to_string(),
            head: "allseeing".to_string(),
            tail: "freckled".to_string(),
            version: "106b".to_string(),
        }
    }

    fn start(&self, _: protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: protocol::Request) -> Result<protocol::MoveResponse, String> {
        let game: logic::Game = req.into();
        let deadline = Instant::now() + game.timeout - MAX_LATENCY;
        println!(
            "request received at {:?}, max duration {}, max latency {}, deadline set at {:?}",
            Instant::now(),
            game.timeout.as_millis(),
            MAX_LATENCY.as_millis(),
            deadline
        );
        let res = look_ahead(&game, &deadline, 1 + num_cpus::get() / 4);

        let (best_dir, max_turns) = res.into_iter().fold((Direction::Down, 0), |acc, entry| {
            if acc.1 > entry.1 {
                acc
            } else {
                entry
            }
        });

        println!("deadline: {:?}, now: {:?}", deadline, Instant::now());

        println!(
            "I think I can survive for at least {} turns when moving {}",
            max_turns, best_dir
        );

        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: "".to_string(),
        })
    }
}
