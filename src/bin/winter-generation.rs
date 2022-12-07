use std::{
    fs::{self},
    io::{Read, Write},
};

use rand::Rng;
use topsnek::logic::scoring::winter;

const REFERENCE_SNAKES: [&str; 1] = ["spaceheater"];
const SNAKE_COUNT: usize = 180;
const CONFIG_DIR: &str = "./cfg";

fn create_new_snake(snake_name: &str) -> bool {
    let filename = format!("{}/{}", CONFIG_DIR, snake_name.to_string());
    let path = std::path::Path::new(&filename);

    if path.exists() {
        return false;
    }

    println!("creating new snake: {}", snake_name);
    let mut file = fs::File::create(path).unwrap();
    file.write_all(b"0").unwrap();
    true
}

fn reset_score(snake_name: &str) {
    let filename = format!("{}/{}", CONFIG_DIR, snake_name.to_string());
    let path = std::path::Path::new(&filename);

    if !path.exists() {
        println!("trying to reset score for missing snake {}", snake_name);
    }

    let mut file = fs::File::open(path).unwrap();
    file.write_all(b"0").unwrap();
}

fn survival_chance(rank: usize) -> f64 {
    0.999f64.powf(rank as f64)
}

fn maybe_kill_snake(snake: &str, rank: usize) -> bool {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0f64..1f64) > survival_chance(rank) {
        println!("killing off snake: {} (rank: {})", snake, rank);
        fs::remove_file(format!("{}/{}", CONFIG_DIR, snake)).unwrap();
        true
    } else {
        false
    }
}

fn maybe_breed_snake(snake: &str, rank: usize) -> Option<String> {
    let mut rng = rand::thread_rng();
    if rng.gen_range(0f64..1f64) < survival_chance(rank) {
        let new_snake = winter::Config::<{ u16::MAX }>::from(snake)
            .evolve()
            .to_string();
        if create_new_snake(&new_snake) {
            println!("creating offspring for snake: {} (rank {})", snake, rank);
            Some(new_snake)
        } else {
            None
        }
    } else {
        None
    }
}

fn main() {
    fs::create_dir_all(CONFIG_DIR).unwrap();

    for snake in REFERENCE_SNAKES {
        if create_new_snake(snake) {
            println!("created reference snake");
        }
    }

    let paths = fs::read_dir(CONFIG_DIR).unwrap();
    let config_count = paths.count();

    if config_count < SNAKE_COUNT {
        println!(
            "Missing snakes: only found {} out of {}",
            config_count, SNAKE_COUNT
        );
        for _ in 0..(SNAKE_COUNT - config_count) {
            while !create_new_snake(&winter::Config::<{ u16::MAX }>::random().to_string()) {}
        }
    }

    let mut scores = vec![];

    let paths = fs::read_dir(CONFIG_DIR).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let snake_name = String::from(path.file_name().unwrap().to_str().unwrap());
        let mut file = fs::File::open(&path).unwrap();
        let mut contents = String::with_capacity(1024);
        file.read_to_string(&mut contents).unwrap();
        let score = contents.parse::<usize>().unwrap();
        scores.push((snake_name, score));
    }

    // Sort by score: top scoring first
    scores.sort_by(|v1, v2| v2.1.cmp(&v1.1));

    let mut snakes: Vec<String> = scores
        .iter()
        .enumerate()
        .filter(|(rank, (snake, _))| maybe_kill_snake(&snake, *rank))
        .map(|(_, (v, _))| v.clone())
        .collect();

    while snakes.len() != SNAKE_COUNT {
        for i in 0..snakes.len() {
            if let Some(new_snake) = maybe_breed_snake(&snakes[i], i) {
                snakes.push(new_snake);
                if snakes.len() == SNAKE_COUNT {
                    break;
                }
            }
        }
    }

    for snake in snakes {
        reset_score(&snake);
    }
}
