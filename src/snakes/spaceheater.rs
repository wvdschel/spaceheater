use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use crate::{
    log,
    logic::{Game, Tile},
    protocol::{self, Direction, Point, ALL_DIRECTIONS},
    util::{thread_count, Scorecard, WorkQueue},
    Battlesnake,
};

struct WorkItem {
    path_so_far: Vec<Direction>,
    game: Game,
}

struct GameSolver {
    work_queues: HashMap<usize, WorkQueue<WorkItem>>,
    scores: Mutex<HashMap<Vec<Direction>, usize>>,
}

fn certain_death(game: &Game, p: &Point, hp: isize) -> bool {
    match game.board.get(p) {
        Tile::Hazard | Tile::HazardWithSnake | Tile::HazardWithHead => {
            game.rules.settings.hazard_damage_per_turn > hp
        }
        Tile::Wall => true,
        // TODO model starvation?
        _ => false,
    }
}

#[allow(unused)]
fn evaluate_game_crowded() -> Vec<Game> {
    // TODO: if there are too many snakes on the board, instead of simulating the other snakes truthfully,
    // simply:
    // - Remove their tails
    // - Turn their head into body
    // - for each neighbouring tile of the old head that does not lead to instant death, copy the snake with the neighbour as its head
    // - make sure copied snakes don't kill each other

    vec![]
}

fn run_games(game: &Game, deadline: &Instant, solver: Arc<GameSolver>) {
    let work_count = Arc::new(AtomicUsize::new(0));
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    {
        let mut sync_queue = queue.lock().unwrap();
        let first_games = evaluate_game(vec![], game, &scores);
        work_count.store(first_games.len(), Ordering::Relaxed);
        for dir_and_game in first_games {
            sync_queue.push_back(dir_and_game);
        }
    }

    let mut threads = vec![];
    for _ in 0..thread_count() {
        let scores = Arc::clone(&scores);
        let queue = Arc::clone(&queue);
        let work_count = Arc::clone(&work_count);
        let deadline = deadline.clone();
        threads.push(thread::spawn(move || {
            loop {
                if Instant::now() > deadline {
                    break;
                }

                let work = {
                    let mut sync_queue = queue.lock().unwrap();
                    sync_queue.pop_front()
                };

                if let Some((initial_dir, game)) = work {
                    let next_games = evaluate_game(vec![initial_dir], &game, &scores);
                    let next_games_count = next_games.len();
                    {
                        let mut sync_queue = queue.lock().unwrap();
                        for dir_and_game in next_games {
                            sync_queue.push_back(dir_and_game);
                        }
                    }
                    loop {
                        let old_count = work_count.load(Ordering::Relaxed);
                        let new_count = old_count + next_games_count - 1;
                        if let Ok(_) = work_count.compare_exchange(
                            old_count,
                            new_count,
                            Ordering::Relaxed,
                            Ordering::Relaxed,
                        ) {
                            break;
                        }
                    }
                } else {
                    // No work received, check if work count is still positive
                    if work_count.load(Ordering::Relaxed) == 0 {
                        log!("out of work");
                        break;
                    }
                    println!("no work received, repolling queue in 2ms");
                    thread::sleep(Duration::from_millis(2));
                }
            }
        }))
    }

    for t in threads {
        t.join().unwrap();
    }
}

fn score_game(game: &Game) -> usize {
    let (turns_alive, kills) = if game.you.health > 0 {
        (game.turn, game.dead_snakes)
    } else {
        (game.turn - 1, game.dead_snakes - 1)
    };

    turns_alive * 100 + kills
}

fn evaluate_game(
    prev_moves: Vec<Direction>,
    game: &Game,
    scores: &Scorecard,
) -> Vec<(Direction, Game)> {
    let mut moves = HashMap::<Direction, Vec<Vec<Direction>>>::new();

    if game.you.health <= 0 {
        log!(
            "warning: asked to evaluate game in which our snake is dead:\n{}",
            game,
        );
        return vec![];
    }

    let mut other_moves: Vec<Vec<Direction>> = vec![vec![]];
    for snake in &game.others {
        let mut viable_directions: Vec<Direction> = ALL_DIRECTIONS
            .into_iter()
            .filter(|&dir| {
                let pos = game.warp(&snake.head.neighbour(dir));
                !certain_death(game, &pos, snake.health)
            })
            .collect();

        if viable_directions.len() == 0 {
            // If all directions lead to death, we do want to add something to prevent this subtree from being ignored.
            viable_directions.push(Direction::Up);
        }

        let mut new_moves = Vec::with_capacity(viable_directions.len() * other_moves.len());
        for dir in viable_directions {
            for old_moves in other_moves.iter() {
                let mut m = old_moves.clone();
                m.push(dir);
                new_moves.push(m);
            }
        }
        other_moves = new_moves;
    }

    for my_dir in ALL_DIRECTIONS {
        let my_pos = game.warp(&game.you.head.neighbour(my_dir));

        if certain_death(game, &my_pos, game.you.health) {
            continue;
        }

        moves.insert(my_dir, other_moves.clone());
    }

    log!(
        "got {} games to evaluate for turn {}",
        moves
            .iter()
            .map(|(_, moves)| { moves.len() })
            .reduce(|sum, move_count| { move_count + sum })
            .unwrap_or_default(),
        game.turn,
    );

    let mut successor_games = vec![];

    for (my_dir, other_moves) in moves {
        let mut min_score = usize::MAX;
        let mut full_path = prev_moves.clone();
        full_path.push(my_dir);
        let full_path = full_path;
        for other_moves in other_moves {
            let mut ngame = game.clone();
            ngame.execute_moves(my_dir, &other_moves);
            let score = score_game(&ngame);
            if score < min_score {
                min_score = score;

                log!(
                    ">>> New min score for {:?}: Turn {} - {} {:?} - {}",
                    full_path,
                    game.turn,
                    my_dir,
                    &other_moves,
                    score,
                );
                log!("BEFORE moving {}: {}", my_dir, game);
                log!("AFTER  moving {}: {}", my_dir, ngame);
            }
            if ngame.you.health > 0 {
                successor_games.push((my_dir, ngame));
            }
        }
        // min_score is now the best score we can get if all other snakes try
        // to minimize our score this turn when moving into my_dir.
        // So post the score to the scoreboard and if it beats our previous best
        // it will become the new top score for this direction
        if scores.post_score(full_path, min_score) != min_score {
            log!(
                ">>>> Direction {}: new min score for turn {}: {} ({:?})",
                full_path.first().unwrap(),
                game.turn,
                min_score,
                full_path,
            );
        }
    }

    successor_games
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
            log!("last turn took {}/{}ms, with {}ms slack for latency. Actual compute time {}, actual latency {}.",
                last_turn_time_ms, max_turn_time_ms, prev_latency_ms, last_turn_compute_time_ms, last_turn_actual_latency);

            if latency_ms > max_turn_time_ms {
                latency_ms = max_turn_time_ms * 10 / 20;
                log!(
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
            color: "#3B224C".to_string(),
            head: "all-seeing".to_string(),
            tail: "freckled".to_string(),
            version: "109b".to_string(),
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
        let scores = Arc::new(Scorecard::new());

        println!(
            "----- request received at {:?}, latency {:?}, deadline set at {:?} -----",
            start_time, latency, deadline
        );

        {
            let req = (*req).clone();
            let scores_clone = scores.clone();
            thread::spawn(move || {
                let game = (&req).into();
                run_games(&game, &deadline, scores_clone);
            });
        }

        let sleep_time = deadline - Instant::now();
        println!("Sleeping for {}ms", sleep_time.as_millis());
        thread::sleep(sleep_time);

        let (best_dir, top_score) = scores.top_score();
        let max_turns = top_score / 100 - req.turn;
        let max_kills = top_score % 100;
        println!(
            "----- Turn {}: I think I can survive for at least {} turns (with {} dead snakes) when moving {} -----\n{}\n{}",
            req.turn, max_turns, max_kills, best_dir, Game::from(req),
            std::iter::repeat("-").take(100).collect::<String>(),
        );

        println!("deadline: {:?}, now: {:?}", deadline, Instant::now());
        Ok(protocol::MoveResponse {
            direction: best_dir,
            shout: "".to_string(),
        })
    }
}
