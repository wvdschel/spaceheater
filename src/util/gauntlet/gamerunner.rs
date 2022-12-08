use std::{thread, time::Duration};

pub fn run_game(args: &Vec<String>, base_url: &str, snake_names: Vec<String>) -> Vec<String> {
    let mut args = args.clone();
    for snake_name in &snake_names {
        args.push(format!("-n\"{}\"", snake_name));
        args.push(format!("-u\"{}/{}/\"", base_url, snake_name));
    }

    // TODO: launch battlesnake cli, parse output
    thread::sleep(Duration::from_secs(1));

    println!("Running: battlesnake {}", args.join(" "));
    snake_names
}
