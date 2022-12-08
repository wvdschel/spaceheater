use std::{
    fs::{self},
    io::{Read, Write},
};

use rand::Rng;
use topsnek::{logic::scoring::winter, util::gauntlet::GeneticConfig};

fn create_new_snake(snake_name: &str) -> bool {
    let filename = format!("{}/{}", CONFIG_DIR, snake_name.to_string());
    let path = std::path::Path::new(&filename);

    if path.exists() {
        return false;
    }

    reset_score(snake_name);

    true
}

fn reset_score(snake_name: &str) {
    let filename = format!("{}/{}", CONFIG_DIR, snake_name.to_string());
    let path = std::path::Path::new(&filename);

    let mut file = fs::File::create(path).unwrap();
    file.write_all(b"0").unwrap();
}

fn breeding_chance(rank: usize) -> f64 {
    let exp = (rank + 1) * 100 / SNAKE_COUNT;
    0.98f64.powf(exp as f64)
}

fn survival_chance(rank: usize) -> f64 {
    let exp = (rank + 1) * 100 / SNAKE_COUNT;
    0.99f64.powf(exp as f64)
}

fn maybe_kill_snake(snake: &str, rank: usize) -> bool {
    for ref_snake in REFERENCE_SNAKES {
        if snake == ref_snake {
            return false;
        }
    }

    let mut rng = rand::thread_rng();
    if rng.gen_range(0f64..1f64) > survival_chance(rank) {
        print!("{} ", rank);
        fs::remove_file(format!("{}/{}", CONFIG_DIR, snake)).unwrap();
        true
    } else {
        false
    }
}

fn maybe_breed_snake(snake: &str, rank: usize) -> Option<String> {
    for ref_snake in REFERENCE_SNAKES {
        if snake == ref_snake {
            return None;
        }
    }

    let mut rng = rand::thread_rng();
    if rng.gen_range(0f64..1f64) < breeding_chance(rank) {
        if let Ok(cfg) = winter::Config::<{ u16::MAX }>::try_from(snake) {
            let new_snake = cfg.evolve().to_string();
            print!("{} ", rank);
            if create_new_snake(&new_snake) {
                return Some(new_snake);
            }
        }
    }
    None
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

    // Sort by score: best scoring first
    scores.sort_by(|v1, v2| v2.1.cmp(&v1.1));

    for (rank, (snake, score)) in scores.iter().enumerate() {
        let mut is_ref = false;
        for ref_snake in REFERENCE_SNAKES {
            if snake == ref_snake {
                is_ref = true;
            }
        }
        if is_ref || rank < 10 {
            println!("#{}: score {}, snake {}", rank, score, snake);
        }
    }

    print!("killing snakes ranked: ");
    let mut snakes: Vec<String> = scores
        .iter()
        .enumerate()
        .filter(|(rank, (snake, _))| !maybe_kill_snake(&snake, *rank))
        .map(|(_, (v, _))| v.clone())
        .collect();
    println!();
    println!("snakes left after deaths: {}", snakes.len());

    print!("breeding snakes ranked: ");
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
    println!();

    for snake in snakes {
        reset_score(&snake);
    }
}
