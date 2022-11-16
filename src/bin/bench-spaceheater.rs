use std::{
    io::stdin,
    time::{Duration, Instant},
};

#[cfg(feature = "profiling")]
use std::fs::File;

use topsnek::{
    logic::{self, scoring, Direction},
    snakes,
    util::gamelogger,
};

fn load_replay() -> gamelogger::Game {
    gamelogger::Game::load(&mut stdin()).unwrap()
}

#[allow(unused)]
fn solve_game(
    game: &logic::Game,
    max_depth: usize,
) -> (Direction, scoring::tournament::TournamentScore) {
    let mut solver = snakes::spaceheater::GameSolver::new(scoring::tournament_score);

    solver.solve(
        format!("solver for depth {}", max_depth).as_str(),
        &game,
        None,
        max_depth,
    )
}

fn solve_game2(
    game: &logic::Game,
    max_depth: usize,
) -> (Direction, scoring::tournament::TournamentScore) {
    let deadline = Instant::now() + Duration::from_secs(100);
    snakes::spaceheater3::solve::solve(
        game.clone(),
        &deadline,
        max_depth,
        &scoring::tournament_score,
    )
    .unwrap()
}

fn main() {
    #[cfg(feature = "profiling")]
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(2000)
        .blocklist(&["libc", "libgcc", "vdso"])
        .build()
        .unwrap();

    let game = logic::Game::from(&load_replay().start_request);

    let mut args = std::env::args();
    args.next();
    let max_iter: usize = args.next().map(|f| f.parse().unwrap_or(6)).unwrap_or(6);

    for d in 3..max_iter {
        let start = Instant::now();
        let (dir, score) = solve_game2(&game, d);
        let stop = Instant::now();
        let duration = stop - start;
        println!(
            "Solved for depth {} in {}ms: {} going {:?}",
            d,
            duration.as_millis(),
            score,
            dir
        );
    }

    #[cfg(feature = "profiling")]
    {
        if let Ok(report) = guard.report().build() {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
        };
    }
}
