use std::collections::HashMap;

use rand::Rng;

#[cfg(test)]
use super::RandomConfig;
#[cfg(test)]
use crate::logic::scoring::winter;
use crate::util::gauntlet::SCORES;

use super::{GeneticConfig, Score};

fn survival_chance(rank: usize, count: usize) -> f64 {
    let exp = (rank + 1) * 100 / count;
    0.99f64.powf(1.2 * exp as f64)
}

fn maybe_kill_snake(rank: usize, count: usize) -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_range(0f64..1f64) > survival_chance(rank, count)
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

    let mut rng = rand::thread_rng();
    while scores.len() + next_gen.len() < target_count {
        let mut snakes = vec![];
        for _ in 0..4 {
            snakes.push(&scores[rng.gen_range(0..scores.len())]);
        }
        snakes.sort_by(|s1, s2| s2.points_per_game().total_cmp(&s1.points_per_game()));
        let snake1 = snakes[0];
        let snake2 = snakes[1];

        if snake1.snake_name != snake2.snake_name {
            let new_name = format!("gen{}_snake{}", generation, snakes_spawned);
            let snake2_genes = snake2.snake_config.unwrap().to_string();

            let weight1 = snake1.points_per_game();
            let weight2 = snake2.points_per_game();
            let ratio_other = weight2 / (weight1 + weight2);

            let mut new_config = if let Some(v) = snake1
                .snake_config
                .unwrap()
                .try_crossover(snake2_genes.as_str(), ratio_other)
            {
                v
            } else {
                continue;
            };

            for _ in 0..rng.gen_range(0..10) {
                new_config = new_config.mutate();
            }
            println!(
                "{} ({}/{} points) and {} ({}/{} points) spawned a new child {} with config {}",
                snake1.snake_name,
                snake1.points,
                snake1.games_played * SCORES[0],
                snake2.snake_name,
                snake2.points,
                snake2.games_played * SCORES[0],
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
    println!();

    for s in scores {
        next_gen.insert(s.snake_name, s.snake_config.unwrap().boxed_clone());
    }

    next_gen
}

#[test]
fn test_new_round() {
    let mut cfgs = Vec::<Box<dyn GeneticConfig>>::new();
    for _ in 0..42 {
        cfgs.push(Box::new(winter::Config::<10>::random()));
    }

    let mut snakes = vec![];
    for i in 0..cfgs.len() {
        snakes.push(Score {
            snake_name: format!("snake_{}", i),
            snake_config: Some(&cfgs[i]),
            points: cfgs.len() - i,
            games_played: 10,
        });
    }

    let next_gen = next_generation(1, &snakes, snakes.len());
    assert_eq!(next_gen.len(), snakes.len())
}
