use thread_priority::*;

use std::{
    collections::HashMap,
    fmt::Display,
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use crate::{
    log,
    logic::{scoring::ApproximateScore, search, BoardLike, Game, Tile},
    protocol::{Direction, Point, ALL_DIRECTIONS},
    util::{thread_count, WorkQueue},
};

use super::scorecard::Scorecard;

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

pub struct GameSolver<T>
where
    T: Ord + Default + Copy + Display + Send,
{
    work_queue: Arc<WorkQueue<WorkItem, usize>>,
    scores: Arc<Scorecard<T>>,
    current_depth: Arc<AtomicUsize>,
    score_fn: fn(&Game) -> T,
}

fn search_for_food(board: &dyn BoardLike, head: &Point, warp: bool) -> Option<(Direction, isize)> {
    let mut food: Option<Point> = None;
    let (w, h) = (board.width(), board.height());
    let distances = search::calculate_distances(
        board,
        head,
        |_, p| {
            let distance = if board.get(p).is_safe() {
                1
            } else {
                isize::MAX
            };
            (
                distance,
                p.neighbours()
                    .map(|(_dir, p)| if warp { p.warp(w, h) } else { p })
                    .into(),
            )
        },
        |_, p| {
            if board.get(&p) == Tile::Food {
                food = Some(p.clone());
                true
            } else {
                false
            }
        },
    );

    if let Some(f) = food {
        if let Some(distance) = distances[f.x as usize][f.y as usize] {
            println!("distance to food at {} is {}", f, distance);
            let path = search::find_path(&distances, head, &f);
            match path.first() {
                Some(v) => return Some((*v, distance)),
                None => println!("can't find route to food, fall back to basic survival"),
            };
        }
    }

    None
}

impl<T: Ord + Default + Copy + Display + Send + ApproximateScore + 'static> GameSolver<T> {
    pub fn new(score_fn: fn(&Game) -> T) -> Self {
        Self {
            work_queue: Arc::new(WorkQueue::new(32 * 1024 * 1024)),
            scores: Arc::new(Scorecard::new()),
            current_depth: Arc::new(AtomicUsize::new(0)),
            score_fn,
        }
    }

    pub fn solve(&mut self, game: &Game, deadline: &Instant) -> (Direction, T) {
        let base_label = move_label!("{}", game);
        let first_games = evaluate_game(vec![], game, self.score_fn, &self.scores, &base_label);
        for work in first_games {
            let priority = usize::MAX - work.path_so_far.len();
            self.work_queue.push(work, priority);
        }

        for _ in 0..thread_count() {
            let scores = Arc::clone(&self.scores);
            let queue = Arc::clone(&self.work_queue);
            let deadline = deadline.clone();
            let current_depth = Arc::clone(&self.current_depth);
            let score_fn = self.score_fn.clone();
            thread::spawn(move || {
                if !set_current_thread_priority(ThreadPriority::Min).is_ok() {
                    println!("warning: failed to change worker thread priority");
                }
                let start_time = Instant::now();
                loop {
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
                                "{}ms: finished depth {} (coming from {}, {} games processed)",
                                (Instant::now() - start_time).as_millis(),
                                depth_finished,
                                old_depth,
                                queue.done_count(),
                            );
                            scores.max_step(depth_finished)
                        } else if depth_finished < old_depth {
                            log!("wtf why are we getting a game for depth {} while {} is supposed to be finished?", depth_finished+1, old_depth);
                        }

                        let next_games = evaluate_game(
                            work.path_so_far,
                            &work.game,
                            score_fn,
                            &scores,
                            &work.label,
                        );
                        for more_work in next_games {
                            let priority = usize::MAX - more_work.path_so_far.len();
                            if !queue.push(more_work, priority) {
                                println!("warning: discarding work because work queue is full");
                            }
                        }
                        queue.done();
                    } else {
                        log!("out of work");
                        break;
                    }
                }
            });
        }

        let sleep_time = *deadline - Instant::now();
        println!("Sleeping for {}ms", sleep_time.as_millis());
        thread::sleep(sleep_time);

        log!("{}", self.scores);

        let (mut top_dir, mut top_score) = self.scores.top_score();

        if let Some((food_dir, food_distance)) = search_for_food(
            game.board.as_ref(),
            &game.you.head,
            game.rules.warped_mode(),
        ) {
            if game.you.health < 40 && food_distance < game.you.health {
                if food_dir != top_dir {
                    let food_score = self.scores.get(&vec![food_dir]);
                    if food_dir != top_dir && food_score.approximate().eq(&top_score.approximate())
                    {
                        println!(
                            "overwriting top score with food score: {} becomes {}",
                            top_dir, food_dir
                        );
                        top_dir = food_dir;
                        top_score = food_score;
                    }
                }
            }
        }
        return (top_dir, top_score);
    }
}

fn evaluate_game<T: Ord + Default + Copy + Display + Send>(
    prev_moves: Vec<Direction>,
    game: &Game,
    score_fn: fn(&Game) -> T,
    scores: &Scorecard<T>,
    _label: &str,
) -> Vec<WorkItem> {
    log!("start eval: {:?} {}", prev_moves, game);
    if scores.is_certain_death(&prev_moves) {
        log!("{:?}: skipping because certain death", prev_moves);
    }
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
        let hitting_my_neck = game.you.length > 1 && (game.you.body[1] == my_pos);
        if hitting_my_neck || certain_death(game, &my_pos, game.you.health) {
            direction_kills_me.insert(my_dir, true);
            continue;
        }

        let mut min_score = Option::<T>::default();
        let mut full_path = prev_moves.clone();
        full_path.push(my_dir);
        let full_path = full_path;
        let mut min_game = String::new();
        for other_moves in other_moves_catalog.iter() {
            let mut ngame = game.clone();
            ngame.execute_moves(my_dir, &other_moves);
            let score = (score_fn)(&ngame);
            if min_score.is_none() || score < min_score.unwrap() {
                log!("{:?}: min score: {} - {}", full_path, score, ngame);
                min_game = move_label!(
                    "{}Enemy moves: {:?} / Score: {}\n{}",
                    _label,
                    other_moves,
                    score,
                    ngame
                );
                min_score = Some(score);
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
                break; // This optimizes our search, at the cost of missing some possibillities to draw.
            }
        }

        // min_score is now the best score we can get if all other snakes try
        // to minimize our score this turn when moving into my_dir.
        // So post the score to the scoreboard and if it beats our previous best
        // it will become the new top score for this direction
        if let Some(min_score) = min_score {
            log!("{:?}: posted score {}", full_path, min_score);
            let _old_score = scores.post_score(full_path, min_score, Some(min_game));
            log!("old score was {}", _old_score);
        }
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

    for (dir, death) in direction_kills_me.iter() {
        if *death {
            let mut path = prev_moves.clone();
            path.push(*dir);
            scores.post_certain_death(path);
        }
    }

    if direction_kills_me.values().all(|b| *b) {
        // All directions lead to death. We might not have posted any scores, but we should post at least one for min-max to work.
        // This indicates we made a mistake a number of turns back that gave the enemy a surefire set of moves to kill us.
        log!("{:?} leads to certain death", prev_moves);
        let mut full_path = prev_moves.clone();
        full_path.push(Direction::Up);
        scores.post_score(
            full_path.clone(),
            (score_fn)(game),
            Some(move_label!("certain death: {}", game)),
        );
        scores.post_certain_death(prev_moves.clone());
    }

    // filter successor games for trees in which another snake always dies
    // no rational snake will move in a direction that leads to certain death,
    // unless all directions lead to death.
    let mut res = vec![];
    for succ in successor_games {
        if direction_kills_me[&succ.my_dir] {
            // At least one set of enemy moves in this direction killed our snake.
            // Discard all games in this branch of the tree.
            scores.post_certain_death(succ.next_state.path_so_far);
            continue;
        }
        let mut rejected = false;
        for (idx, _snake) in game.others.iter().enumerate() {
            let snake_dir = succ.other_moves[idx];
            let directions_snake_can_survive = directions_others_can_survive.get(idx).unwrap();
            if !directions_snake_can_survive
                .get(&snake_dir)
                .unwrap_or(&false)
            {
                rejected = true;
                // log!("rejected by {}:\n{}", _snake, succ.next_state.game);
                break;
            }
        }
        if rejected {
            continue;
        }
        // log!(
        //     "{:?}: successor state: {} [{:?}]\n{}",
        //     prev_moves,
        //     succ.my_dir,
        //     succ.other_moves,
        //     succ.next_state.game
        // );
        res.push(succ.next_state);
    }
    res
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

fn all_possible_enemy_moves(game: &Game) -> Vec<Vec<Direction>> {
    let mut other_moves: Vec<Vec<Direction>> = vec![vec![]];
    for snake in &game.others {
        let mut viable_directions: Vec<Direction> = ALL_DIRECTIONS
            .into_iter()
            .filter(|&dir| {
                let pos = game.warp(&snake.head.neighbour(dir));
                let mut survives = !certain_death(game, &pos, snake.health);
                if snake.length > 1 {
                    // Don't move into your own neck
                    survives &= !(snake.body[1] == pos);
                }
                survives
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
