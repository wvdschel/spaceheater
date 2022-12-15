use std::collections::HashMap;

use rand::Rng;

#[cfg(test)]
use super::RandomConfig;
#[cfg(test)]
use crate::logic::scoring::winter;

use super::{GeneticConfig, Score};

fn breeding_chance(rank: usize, count: usize) -> f64 {
    let exp = (rank + 1) * 100 / count;
    0.985f64.powf(exp as f64)
}

fn survival_chance(rank: usize, count: usize) -> f64 {
    let exp = (rank + 1) * 100 / count;
    0.99f64.powf(1.2 * exp as f64)
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
    let scores: Vec<Score> = scores
        .iter()
        .filter(|s| s.snake_config.is_some())
        .enumerate()
        .filter(|(rank, score)| {
            if maybe_kill_snake(*rank, scores.len()) {
                println!(
                    "killing snake {} (ranked #{}, {} points)",
                    score.snake_name, rank, score.points
                );
                false
            } else {
                true
            }
        })
        .map(|(_, score)| score.clone())
        .collect();
    println!();
    println!("snakes left after deaths: {}", scores.len());

    let mut snakes_spawned = 0;
    let mut next_gen: HashMap<String, Box<dyn GeneticConfig>> = HashMap::new();
    while scores.len() + next_gen.len() < target_count {
        for (rank, score) in scores.iter().enumerate() {
            if maybe_breed_snake(rank, scores.len()) {
                let new_name = format!("gen{}_snake{}", generation, snakes_spawned);
                let mut new_config = score.snake_config.unwrap().boxed_clone();
                for _ in 0..12 {
                    new_config = new_config.mutate();
                }
                println!(
                    "{} (rank #{}, {} points) spawned a new child {} with config {}",
                    score.snake_name,
                    rank,
                    score.points,
                    new_name,
                    new_config.to_string()
                );
                snakes_spawned += 1;
                next_gen.insert(new_name, new_config);
                if scores.len() + next_gen.len() == target_count {
                    break;
                }
            }
        }
        println!(
            "snakes after breeding: {} + {}",
            scores.len(),
            next_gen.len(),
        )
    }
    println!();

    for s in scores {
        next_gen.insert(s.snake_name, s.snake_config.unwrap().boxed_clone());
    }

    next_gen
}

#[test]
fn test_new_round() {
    let mut cfgs = Vec::<Box<dyn GeneticConfig>>::new();
    for _ in 0..100 {
        cfgs.push(Box::new(winter::Config::<10>::random()));
    }

    let mut snakes = vec![];
    for i in 0..cfgs.len() {
        snakes.push(Score {
            snake_name: format!("snake_{}", i),
            snake_config: Some(&cfgs[i]),
            points: 0,
        });
    }

    let next_gen = next_generation(1, &snakes, snakes.len());
    assert_eq!(next_gen.len(), snakes.len())
}
