use std::{io::stdin, time::Instant};

#[cfg(feature = "profiling")]
use std::fs::File;

use topsnek::{logic::scoring, snakes, util::gamelogger, Battlesnake};

fn load_replay() -> gamelogger::Game {
    gamelogger::Game::load(&mut stdin()).unwrap()
}

fn main() {
    #[cfg(feature = "profiling")]
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(2000)
        .blocklist(&["libc", "libgcc", "vdso"])
        .build()
        .unwrap();

    let replay = load_replay();

    let snake = snakes::Spaceheater3::new(scoring::tournament_score, None);

    let mut args = std::env::args();
    args.next();
    let first_turn: usize = args
        .next()
        .map(|f| f.parse().unwrap_or(0))
        .unwrap_or(0)
        .min(replay.moves.len() - 1);
    let last_turn: usize = args
        .next()
        .map(|f| f.parse().unwrap_or(replay.moves.len()))
        .unwrap_or(replay.moves.len())
        .min(replay.moves.len());

    let start_request = &replay.start_request;
    println!(
        "running game {}: {}, {} snakes, {}x{}, turn {} up to {}",
        start_request.game.id,
        start_request.game.map,
        start_request.board.snakes.len(),
        start_request.board.width,
        start_request.board.height,
        first_turn,
        last_turn
    );
    for (req, _) in &replay.moves[first_turn..last_turn] {
        let start = Instant::now();
        let res = snake.make_move(req);
        let duration = start.elapsed();
        println!(
            "solved turn {} in {} ms: {}",
            req.turn,
            duration.as_millis(),
            res.map_or("error".to_string(), |r| format!(
                "{} {}",
                r.direction, r.shout
            )),
        );
    }
    println!("{}", "=".repeat(80));

    #[cfg(feature = "profiling")]
    {
        if let Ok(report) = guard.report().build() {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
        };
    }
}
