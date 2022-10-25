use std::{fs, time::Instant};

use crate::{
    logic::{self, scoring, Direction},
    snakes,
    util::gamelogger,
};

fn load_replays() -> Vec<gamelogger::Game> {
    let mut res = vec![];

    for replay in fs::read_dir("./benches/gamedata").unwrap() {
        let replay = replay.unwrap();
        let mut file = fs::File::open(replay.path()).unwrap();
        let game = gamelogger::Game::load(&mut file).unwrap();
        res.push(game);
    }

    res
}

fn solve_game(
    game: &logic::Game,
    max_depth: usize,
) -> (Direction, scoring::TournamentVoronoiScore) {
    let mut solver = snakes::spaceheater::GameSolver::new(scoring::tournament_voronoi);
    println!("Attempting to solve for depth {}: {}", max_depth, game);
    solver.solve(
        format!("solver for depth {}", max_depth).as_str(),
        &game,
        None,
        max_depth,
    )
}

#[test]
fn test_solver() {
    let replays = load_replays();
    let game = logic::Game::from(&replays[0].start_request);

    for d in 1..4 {
        let start = Instant::now();
        let (dir, score) = solve_game(&game, d);
        let stop = Instant::now();
        let duration = stop - start;
        println!(
            "Solved for depth {} in {}ms: {} going {}",
            d,
            duration.as_millis(),
            score,
            dir
        );
    }
}
