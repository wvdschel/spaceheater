use std::{collections::HashMap, time::SystemTime};

use crate::{snakes, Battlesnake};

mod gamerunner;
mod generation;
mod pairing;
mod report;

const SCORES: [isize; 4] = [40, -10, -12, -14];

pub trait RandomConfig {
    fn random() -> Self;
}

pub trait GeneticConfig: ToString {
    fn load(&mut self, cfg: &str);
    fn evolve(&self) -> Box<dyn GeneticConfig>;
    fn battlesnake(&self) -> Box<dyn Battlesnake + Sync + Send>;
}

pub struct Gauntlet {
    configs: HashMap<String, Box<dyn GeneticConfig>>,
    generation: usize,
    snake_count: usize,
    battlesnake_cli_args: Vec<String>,
}

impl Gauntlet {
    pub fn new(cli_args: &[&str]) -> Self {
        Self {
            configs: HashMap::new(),
            generation: 0,
            snake_count: 0,
            battlesnake_cli_args: cli_args.iter().map(|v| v.to_string()).collect(),
        }
    }

    pub fn add_contestant<T: GeneticConfig + 'static>(&mut self, cfg: T) {
        self.configs.insert(
            format!("snake{}_gen{}", self.generation, self.snake_count),
            Box::new(cfg),
        );
        self.snake_count += 1;
    }

    pub fn generate_contestants<T: GeneticConfig + RandomConfig + 'static>(
        &mut self,
        count: usize,
    ) {
        for _ in 0..count {
            self.add_contestant(T::random());
        }
    }

    fn reference_snakes() -> HashMap<String, Box<dyn Battlesnake + Sync + Send>> {
        snakes::snakes()
    }

    fn all_snake_names(&self) -> Vec<String> {
        let mut all_snake_names = Vec::from(
            self.configs
                .keys()
                .map(|v| v.clone())
                .collect::<Vec<String>>(),
        );
        for k in Self::reference_snakes().keys() {
            all_snake_names.push(k.clone());
        }
        all_snake_names
    }

    fn new_round(&self) {
        let all_snake_names = self.all_snake_names();
        let games = pairing::generate_pairings(&all_snake_names);
        let mut scores = HashMap::new();
        for snake in all_snake_names {
            scores.insert(snake, 0 as isize);
        }

        for (idx, game) in games.iter().enumerate() {
            println!(
                "Running game {}/{}: {}",
                idx,
                games.len(),
                game.join(" vs ")
            );
            let mut snakes: HashMap<_, _> = Self::reference_snakes()
                .into_iter()
                .filter(|(name, _)| game.contains(name))
                .collect();
            for snake_name in game {
                if let Some(cfg) = self.configs.get(snake_name) {
                    snakes.insert(snake_name.clone(), cfg.battlesnake());
                }
            }
            let result = gamerunner::run_game(&self.battlesnake_cli_args, snakes);

            for (rank, snake) in result.into_iter().enumerate() {
                let s = scores.get_mut(&snake).unwrap();
                *s += SCORES[rank];
                println!(
                    "#{}: {}, points {}, new score is {}",
                    rank, snake, SCORES[rank], *s
                );
            }
        }
    }
}
