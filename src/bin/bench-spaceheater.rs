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

fn solve_game(
    game: &logic::Game,
    max_depth: usize,
) -> (Direction, scoring::tournament::TournamentScore) {
    let deadline = Instant::now() + Duration::from_millis(60000);
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

    let start_request = load_replay().start_request;
    println!(
        "running game {}: {}, {} snakes, {}x{}",
        start_request.game.id,
        start_request.game.map,
        start_request.board.snakes.len(),
        start_request.board.width,
        start_request.board.height,
    );

    let game = logic::Game::from(&start_request);

    let mut args = std::env::args();
    args.next();
    let max_depth: usize = args.next().map(|f| f.parse().unwrap_or(5)).unwrap_or(5);

    let start = Instant::now();
    let (dir, score) = solve_game(&game, max_depth);
    let duration = start.elapsed();
    println!(
        "Solved for depth {} in {}ms: {} going {:?}",
        max_depth,
        duration.as_millis(),
        score,
        dir
    );

    #[cfg(feature = "profiling")]
    {
        if let Ok(report) = guard.report().build() {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
        };
    }
}
