use std::{io::stdin, time::Instant};

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
) -> (Direction, scoring::TournamentVoronoiScore) {
    let mut solver = snakes::spaceheater::GameSolver::new(scoring::tournament_voronoi);

    solver.solve(
        format!("solver for depth {}", max_depth).as_str(),
        &game,
        None,
        max_depth,
    )
}

fn main() {
    #[cfg(feature = "profiling")]
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "vdso"])
        .build()
        .unwrap();

    let game = logic::Game::from(&load_replay().start_request);

    let mut args = std::env::args();
    let max_iter: usize = args.next().map(|f| f.parse().unwrap_or(4)).unwrap_or(4);

    for d in 1..max_iter {
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

    #[cfg(feature = "profiling")]
    {
        if let Ok(report) = guard.report().build() {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
            println!("report: {:?}", &report);
        };
    }
}
