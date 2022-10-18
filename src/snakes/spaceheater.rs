use std::{
    collections::HashMap,
    hash::Hash,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
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

#[cfg(feature = "logging")]
macro_rules! move_label {
    ($($arg:tt)*) => {{
        let res = format!($($arg)*);
        res
    }}
}

#[macro_export]
#[cfg(not(feature = "logging"))]
macro_rules! move_label {
    ( $( $x:tt )* ) => {
        String::new()
    };
}

#[derive(Hash, Eq, PartialEq)]
struct WorkItem {
    path_so_far: Vec<Direction>,
    game: Game,
    label: String,
}

struct GameSolver {
    work_queue: Arc<WorkQueue<WorkItem, usize>>,
    scores: Arc<Scorecard<usize>>,
    current_depth: Arc<AtomicUsize>,
}

impl GameSolver {
    fn new() -> Self {
        Self {
            work_queue: Arc::new(WorkQueue::new()),
            scores: Arc::new(Scorecard::new()),
            current_depth: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn solve(&mut self, game: &Game, deadline: &Instant) -> (Direction, usize) {
        let base_label = move_label!("{}", game);
        let first_games = evaluate_game(vec![], game, &self.scores, &base_label);
        for work in first_games {
            let priority = usize::MAX - work.path_so_far.len();
            self.work_queue.push(work, priority);
        }
        for _ in 0..thread_count() {
            let scores = Arc::clone(&self.scores);
            let queue = Arc::clone(&self.work_queue);
            let deadline = deadline.clone();
            let current_depth = Arc::clone(&self.current_depth);
            thread::spawn(move || loop {
                if Instant::now() > deadline {
                    break;
                }

                if let Some(work) = queue.pop() {
                    if Instant::now() > deadline {
                        break;
                    }
                    let depth_finished = work.path_so_far.len() - 1;
                    let old_depth = current_depth.swap(depth_finished, Ordering::Relaxed);
                    if depth_finished > old_depth {
                        println!(
                            "{:?} finished depth {} (coming from {})",
                            Instant::now(),
                            depth_finished,
                            old_depth
                        );
                        scores.max_step(depth_finished)
                    } else if depth_finished < old_depth {
                        log!("wtf why are we getting a game for depth {} while {} is supposed to be finished?", depth_finished+1, old_depth);
                    }

                    let next_games =
                        evaluate_game(work.path_so_far, &work.game, &scores, &work.label);
                    for more_work in next_games {
                        let priority = usize::MAX - more_work.path_so_far.len();
                        queue.push(more_work, priority);
                    }
                    queue.done();
                } else {
                    log!("out of work");
                    break;
                }
            });
        }

        let sleep_time = *deadline - Instant::now();
        println!("Sleeping for {}ms", sleep_time.as_millis());
        thread::sleep(sleep_time);

        log!("{}", self.scores);
        return self.scores.top_score();
    }
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

fn score_game(game: &Game) -> usize {
    let (turns_alive, kills) = if game.you.health > 0 {
        (game.turn, game.dead_snakes)
    } else {
        (game.turn - 1, game.dead_snakes - 1)
    };

    turns_alive * 10000 + kills * 100 + game.you.length
}

fn all_possible_enemy_moves(game: &Game) -> Vec<Vec<Direction>> {
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
            log!("{} will die anyway, it's going up:\n{}", snake, game);
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
    other_moves
}

fn evaluate_game(
    prev_moves: Vec<Direction>,
    game: &Game,
    scores: &Scorecard<usize>,
    _label: &str,
) -> Vec<WorkItem> {
    if game.you.health <= 0 {
        println!(
            "warning: asked to evaluate game in which our snake is dead:\n{}",
            game,
        );
        return vec![];
    }

    let other_moves_catalog = all_possible_enemy_moves(game);

    struct Successor {
        my_dir: Direction,
        other_moves: Vec<Direction>,
        next_state: WorkItem,
    }

    let mut successor_games: Vec<Successor> = vec![];
    let mut direction_kills_me = HashMap::from(ALL_DIRECTIONS.map(|d| (d, false)));
    let mut directions_others_can_survive =
        Vec::<HashMap<Direction, bool>>::with_capacity(game.others.len());
    directions_others_can_survive.resize(
        game.others.len(),
        HashMap::from(ALL_DIRECTIONS.map(|d| (d, false))),
    );

    for my_dir in ALL_DIRECTIONS {
        // Eliminate directions which would lead to certain death
        let my_pos = game.warp(&game.you.head.neighbour(my_dir));
        if certain_death(game, &my_pos, game.you.health) {
            continue;
        }

        let mut min_score = usize::MAX;
        let mut full_path = prev_moves.clone();
        full_path.push(my_dir);
        let full_path = full_path;
        let mut min_game = String::new();
        for other_moves in other_moves_catalog.iter() {
            let mut ngame = game.clone();
            ngame.execute_moves(my_dir, &other_moves);
            let score = score_game(&ngame);
            if score < min_score {
                min_game = move_label!(
                    "{}Enemy moves: {:?} / Score: {}\n{}",
                    _label,
                    other_moves,
                    score,
                    ngame
                );
                min_score = score;
            }

            for (i, other) in ngame.others.iter().enumerate() {
                if other.health > 0 {
                    let dir = other_moves[i];
                    directions_others_can_survive[i].insert(dir, true);
                }
            }

            if ngame.you.health > 0 {
                let label = move_label!(
                    "{}Enemy moves: {:?} / Score: {}\n{}",
                    _label,
                    other_moves,
                    score,
                    ngame
                );
                successor_games.push(Successor {
                    my_dir,
                    other_moves: other_moves.clone(),
                    next_state: WorkItem {
                        path_so_far: full_path.clone(),
                        game: ngame,
                        label,
                    },
                });
            } else {
                // Any direction in which the enemies have a combination of moves that would lead to our
                // death needs to be avoided
                direction_kills_me.insert(my_dir, true);
            }
        }

        // min_score is now the best score we can get if all other snakes try
        // to minimize our score this turn when moving into my_dir.
        // So post the score to the scoreboard and if it beats our previous best
        // it will become the new top score for this direction
        log!("{:?}: posted score {}", full_path, min_score);
        let _old_score = scores.post_score_with_label(full_path, min_score, Some(min_game));
        log!("old score was {}", _old_score);
    }

    for survival_map in directions_others_can_survive.iter_mut() {
        // If all directions would lead to a snake dieing, then don't use that snake to
        // filter in the next step.
        if survival_map.values().all(|b| *b) {
            for dir in ALL_DIRECTIONS {
                survival_map.insert(dir, true);
            }
        }
    }

    // filter successor games for trees in which another snake always dies
    // no rational snake will move in a direction that leads to certain death,
    // unless all directions lead to death.
    let mut res = vec![];
    for succ in successor_games {
        if direction_kills_me[&succ.my_dir] {
            continue;
        }
        let mut rejected = false;
        for (idx, _) in game.others.iter().enumerate() {
            let snake_dir = succ.other_moves[idx];
            let directions_snake_can_survive = directions_others_can_survive.get(idx).unwrap();
            if !directions_snake_can_survive
                .get(&snake_dir)
                .unwrap_or(&false)
            {
                rejected = true;
                break;
            }
        }
        if rejected {
            continue;
        }
        res.push(succ.next_state);
    }
    res
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

            // 120% + 25 seems like a sensible margin for ping fluctuations
            latency_ms = last_turn_actual_latency * 12 / 10 + 25;
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

        println!(
            "----- request received at {:?}, latency {:?}, deadline set at {:?} -----",
            start_time, latency, deadline
        );

        let (best_dir, top_score) = GameSolver::new().solve(&req.into(), &deadline);

        let top_score = top_score / 100;
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
