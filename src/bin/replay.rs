use std::{io::stdin, thread, time::Duration};

use topsnek::{util::gamelogger, *};

fn main() {
    let snakes = snakes::snakes();
    let mut args = std::env::args();

    let cmd_name = args.next().unwrap_or("replay".into());
    let snake_name = match args.next() {
        Some(v) => v,
        None => {
            println!("usage: {} <snakename>", cmd_name);
            std::process::exit(1);
        }
    };

    if !snakes.contains_key(&snake_name) {
        println!("unknown snake {}", snake_name);
        std::process::exit(1);
    }

    let snake = snakes.get(&snake_name).unwrap();

    match gamelogger::Game::load(&mut stdin()) {
        Ok(game) => game.replay(snake.as_ref()),
        Err(e) => {
            println!("failed to load game: {}", e);
            std::process::exit(1);
        }
    }

    println!("sleeping 1s to let worker threads finish logging :)");
    thread::sleep(Duration::from_millis(1000));
}
