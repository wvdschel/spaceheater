use std::collections::HashMap;

use rand::Rng;

use super::{GeneticConfig, Score};

fn breeding_chance(rank: usize, count: usize) -> f64 {
    let exp = (rank + 1) * 100 / count;
    0.98f64.powf(exp as f64)
}

fn survival_chance(rank: usize, count: usize) -> f64 {
    let exp = (rank + 1) * 100 / count;
    0.99f64.powf(exp as f64)
}

fn maybe_kill_snake(rank: usize, count: usize) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0f64..1f64) > survival_chance(rank, count)
}

fn maybe_breed_snake(rank: usize, count: usize) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0f64..1f64) < breeding_chance(rank, count)
}

pub fn next_generation(
    generation: usize,
    scores: &Vec<Score>,
    target_count: usize,
) -> HashMap<String, Box<dyn GeneticConfig>> {
    let mut next_gen: HashMap<String, Box<dyn GeneticConfig>> = scores
        .iter()
        .filter(|s| s.snake_config.is_some())
        .enumerate()
        .filter(|(rank, score)| {
            if maybe_kill_snake(*rank, scores.len()) {
                println!("killing snake {} (ranked #{})", score.snake_name, rank);
                false
            } else {
                true
            }
        })
        .map(|(_, score)| {
            (
                score.snake_name.clone(),
                score.snake_config.unwrap().boxed_clone(),
            )
        })
        .collect();
    println!();
    println!("snakes left after deaths: {}", next_gen.len());

    let mut snakes_spawned = 0;
    while next_gen.len() != target_count {
        let mut tmp: HashMap<String, Box<dyn GeneticConfig>> = HashMap::new();
        for (rank, (snake_name, cfg)) in next_gen.iter().enumerate() {
            if maybe_breed_snake(rank, scores.len()) {
                let new_name = format!("gen{}_snake{}", generation, snakes_spawned);
                let new_config = cfg.evolve();
                println!(
                    "{} (rank #{}) spawned a new child {} with config {}",
                    snake_name,
                    rank,
                    new_name,
                    new_config.to_string()
                );
                snakes_spawned += 1;
                tmp.insert(new_name, new_config);
                if next_gen.len() + tmp.len() == target_count {
                    break;
                }
            }
        }
        for (n, c) in tmp.into_iter() {
            next_gen.insert(n, c);
        }
    }
    println!();

    next_gen
}
