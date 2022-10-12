use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    logic::Game,
    protocol::{self, ALL_DIRECTIONS},
    util::Scorecard,
    Battlesnake,
};

fn look_ahead(game: &Game, deadline: &Instant, fork: usize, scores: &Scorecard) {
    if fork > 0 {
        fork_lookahead(game, deadline, fork - 1, scores)
    } else {
        simple_lookahead(game, deadline, scores)
    }
}

fn fork_lookahead(game: &Game, deadline: &Instant, _fork: usize, scores: &Scorecard) {
    // TODO fork 'n shit
    simple_lookahead(game, deadline, scores)
}

fn simple_lookahead(game: &Game, deadline: &Instant, scores: &Scorecard) {
    if &Instant::now() > deadline {
        return;
    }
    let start_turn = game.turn;
    let mut queue = VecDeque::new();
    for d in ALL_DIRECTIONS {
        let mut ng = game.clone();
        ng.execute_moves(d, vec![]);
        if ng.you.health > 0 {
            // println!("initial move to {} succeeds!", d);
            // println!(
            //     "===== BEFORE =====\n{}\n===== AFTER  =====\n{}\n==================",
            //     game.board, ng.board
            // );
            scores.post_score(d, 1);
            queue.push_back((d, ng));
        }
    }

    // println!("======== TURN {} ========", game.turn);
    // println!("hp = {}", game.you.health);
    // println!("{}", game.board);

    while &Instant::now() < deadline {
        if let Some((first_dir, game)) = queue.pop_front() {
            for dir in ALL_DIRECTIONS {
                let mut ng = game.clone();
                let score = ng.turn - start_turn;
                ng.execute_moves(dir, vec![]);
                // if score == 1 {
                //     println!("----- BEFORE -----");
                //     println!("hp = {}", game.you.health);
                //     println!("{}", game.board);

                //     println!("----- TURN {}: GO {} -----", ng.turn, dir);
                //     println!("hp = {}", ng.you.health);
                //     println!("{}", ng.board);
                // }
                if ng.you.health > 0 {
                    scores.post_score(first_dir, score);
                    queue.push_back((first_dir, ng))
                }
            }
        } else {
            println!("reached end of game tree");
            break;
        }
    }
}

#[derive(Clone)]
pub struct SpaceHeater {
    last_turn_latency_estimate: Arc<AtomicU64>,
}

impl SpaceHeater {
    pub fn new() -> Self {
        Self {
            last_turn_latency_estimate: Arc::new(AtomicU64::new(55)),
        }
    }

    fn calculate_latency(&self, last_turn_time_ms: u64, max_turn_time_ms: u64) -> Duration {
        let prev_latency_ms = self.last_turn_latency_estimate.load(Ordering::Acquire);
        let mut latency_ms = prev_latency_ms;
        if last_turn_time_ms > prev_latency_ms {
            let last_turn_compute_time_ms = max_turn_time_ms - prev_latency_ms;
            let last_turn_actual_latency = last_turn_time_ms - last_turn_compute_time_ms;

            // 120% + 1 seems like a sensible margin for ping fluctuations
            latency_ms = last_turn_actual_latency * 12 / 10 + 1;
            println!("last turn took {}/{}ms, with {}ms slack for latency. Actual compute time {}, actual latency {}.",
                last_turn_time_ms, max_turn_time_ms, prev_latency_ms, last_turn_compute_time_ms, last_turn_actual_latency);

            if latency_ms > max_turn_time_ms {
                latency_ms = max_turn_time_ms * 10 / 20;
                println!(
                    "estimated latency exceeds turn time - limiting to {}ms",
                    latency_ms
                );
            }
        }
        self.last_turn_latency_estimate
            .store(latency_ms, Ordering::Release);
        Duration::from_millis(latency_ms)
    }
}

impl Battlesnake for SpaceHeater {
    fn snake_info(&self) -> protocol::SnakeInfo {
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "General Error".to_string(),
            color: "#4A0E3D".to_string(),
            head: "all-seeing".to_string(),
            tail: "freckled".to_string(),
            version: "106b".to_string(),
        }
    }

    fn start(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: &protocol::Request) -> Result<protocol::MoveResponse, String> {
        let last_turn_duration_ms = req.you.latency.parse::<u64>().unwrap_or(0);
        let max_turn_time_ms = req.game.timeout as u64;
        let latency = self.calculate_latency(last_turn_duration_ms, max_turn_time_ms);
        let start_time = Instant::now();
        let deadline = start_time + Duration::from_millis(max_turn_time_ms) - latency;

        println!(
            "request received at {:?}, latency {:?}, deadline set at {:?}",
            start_time, latency, deadline
        );
        let scores = Arc::new(Scorecard::new());

        let req = (*req).clone();
        let scores_clone = scores.clone();
        thread::spawn(move || {
            let game = (&req).into();
            look_ahead(&game, &deadline, 1 + num_cpus::get() / 4, &scores_clone);
        });

        let sleep_time = deadline - start_time;
        println!("Sleeping for {}ms", sleep_time.as_millis());
        thread::sleep(sleep_time);

        let (best_dir, max_turns) = scores.top_score();
        println!(
            "I think I can survive for at least {} turns when moving {}",
            max_turns, best_dir
        );

        println!("deadline: {:?}, now: {:?}", deadline, Instant::now());
        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: "".to_string(),
        })
    }
}
