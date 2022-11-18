use std::{io::stdin, thread, time::Duration};

#[cfg(feature = "profiling")]
use std::fs::File;

use topsnek::{util::gamelogger, *};

fn main() {
    let snakes = snakes::snakes();
    let mut args = std::env::args();

    let cmd_name = args.next().unwrap_or("replay".into());
    let snake_name = match args.next() {
        Some(v) => v,
        None => {
            println!(
                "usage: {} <snakename> [<start_turn> <end_turn> <millis_per_move>]",
                cmd_name
            );
            std::process::exit(1);
        }
    };

    let start_turn: Option<usize> = args.next().map(|f| f.parse().unwrap());
    let end_turn: Option<usize> = args.next().map(|f| f.parse().unwrap());
    let millis: Option<usize> = args.next().map(|f| f.parse().unwrap());

    if !snakes.contains_key(&snake_name) {
        println!("unknown snake {}", snake_name);
        std::process::exit(1);
    }

    let snake = snakes.get(&snake_name).unwrap();

    match gamelogger::Game::load(&mut stdin()) {
        Ok(game) => {
            #[cfg(feature = "profiling")]
            let guard = pprof::ProfilerGuardBuilder::default()
                .frequency(2000)
                .blocklist(&["libc", "libgcc", "vdso"])
                .build()
                .unwrap();

            game.replay(snake.as_ref(), start_turn, end_turn, millis);

            #[cfg(feature = "profiling")]
            {
                if let Ok(report) = guard.report().build() {
                    let file = File::create(format!(
                        "flamegraph_{}_{}.svg",
                        snake_name, game.start_request.game.id
                    ))
                    .unwrap();
                    report.flamegraph(file).unwrap();
                };
            }
        }
        Err(e) => {
            println!("failed to load game: {}", e);
            std::process::exit(1);
        }
    }

    println!("sleeping 10s to let worker threads finish logging :)");
    thread::sleep(Duration::from_millis(10000));
}
