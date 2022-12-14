use std::{
    collections::HashMap,
    sync::{mpsc::channel, Arc, Mutex},
    thread,
    time::Instant,
};

use crate::{snakes, util::gauntlet::webserver::Webserver, Battlesnake};

use self::{generation::next_generation, report::write_report};

mod gamerunner;
mod generation;
mod names;
mod pairing;
mod report;
mod webserver;

const SCORES: [usize; 4] = [4, 2, 1, 0];

pub trait RandomConfig {
    fn random() -> Self;
}

pub trait GeneticConfig: ToString + Sync + Send {
    fn load(&mut self, cfg: &str);
    fn try_crossover(&self, other_genes: &str, ratio_other: f64) -> Option<Box<dyn GeneticConfig>>;
    fn mutate(&self) -> Box<dyn GeneticConfig>;
    fn battlesnake(&self) -> Box<dyn Battlesnake + Sync + Send>;
    fn boxed_clone(&self) -> Box<dyn GeneticConfig>;
}

pub struct Gauntlet {
    configs: HashMap<String, Box<dyn GeneticConfig>>,
    generation: usize,
    battlesnake_cli_args: Vec<String>,
}

#[derive(Clone)]
pub struct Score<'a> {
    pub snake_name: String,
    pub snake_config: Option<&'a Box<dyn GeneticConfig>>,
    pub points: usize,
    pub games_played: usize,
}

impl<'a> Score<'a> {
    fn points_per_game(&self) -> f64 {
        if self.games_played == 0 {
            return 0.0;
        }
        self.points as f64 / self.games_played as f64
    }
}

impl Gauntlet {
    pub fn new(cli_args: &[&str]) -> Self {
        Self {
            configs: HashMap::new(),
            generation: 0,
            battlesnake_cli_args: cli_args.iter().map(|v| v.to_string()).collect(),
        }
    }

    pub fn contestant_count(&self) -> usize {
        self.configs.len()
    }

    pub fn add_contestant<T: GeneticConfig + 'static>(&mut self, name: &str, cfg: T) {
        let mut actual_name = name.to_string();
        let mut num = 1;
        while self.configs.contains_key(&actual_name) {
            num += 1;
            actual_name = format!("{} ({})", name, num);
        }

        self.configs.insert(actual_name, Box::new(cfg));
    }

    pub fn generate_contestants<T: GeneticConfig + RandomConfig + 'static>(
        &mut self,
        count: usize,
    ) {
        for _ in 0..count {
            self.add_contestant(
                format!("starter_gen{}_snake{}", self.generation, self.configs.len()).as_str(),
                T::random(),
            );
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

    pub fn new_round(&mut self, matches_per_pair: usize, concurrent_games: usize) {
        let all_snake_names = self.all_snake_names();
        let mut games = vec![];
        for _ in 0..matches_per_pair {
            games.append(&mut pairing::generate_pairings(&all_snake_names, 4));
        }
        let mut scores = HashMap::new();
        let mut sorted_scores = vec![];
        for snake in &all_snake_names {
            scores.insert(
                snake,
                Score {
                    snake_name: snake.clone(),
                    snake_config: self.configs.get(snake),
                    points: 0,
                    games_played: 0,
                },
            );
        }
        let game_count = games.len();
        let (result_tx, result_rx) = channel();

        let mut snakes: HashMap<_, _> = Self::reference_snakes();
        for (snake_name, cfg) in &self.configs {
            snakes.insert(snake_name.clone(), cfg.battlesnake());
        }
        let server = Webserver::new(snakes);

        println!(
            "Starting generation {}, running {} games",
            self.generation, game_count
        );
        let games = Arc::new(Mutex::new(games));
        let mut threads = vec![];
        for t in 0..concurrent_games {
            let base_url = server.address().to_string();
            let result_tx = result_tx.clone();
            let cli_args = self.battlesnake_cli_args.clone();
            let games = games.clone();
            threads.push(thread::spawn(move || {
                loop {
                    let game = {
                        let mut l = games.lock().unwrap();
                        let g = l.pop();
                        drop(l);
                        g
                    };
                    if game == None {
                        break;
                    }
                    let game = game.unwrap();

                    println!("Running game: {}", game.join(" vs "));
                    let result = gamerunner::run_game(&cli_args, base_url.as_str(), game);
                    result_tx.send(result).unwrap();
                }
                println!("games finished, thread {} returning", t);
            }));
        }
        drop(result_tx);

        let start_time = Instant::now();
        let mut result_count = 0;
        while let Ok(result) = result_rx.recv() {
            result_count += 1;
            println!("Received results for game {}/{}", result_count, game_count);
            for (rank, snake) in result.into_iter().enumerate() {
                let s = scores.get_mut(&snake).unwrap();
                s.points += SCORES[rank];
                s.games_played += 1;
                println!(
                    "#{}: {}, points {}, new score is {}",
                    rank + 1,
                    snake,
                    SCORES[rank],
                    s.points
                );
            }
            let time_elapsed = start_time.elapsed();
            let avg_time_per_game = time_elapsed.as_secs_f32() / result_count as f32;
            let seconds_remaining = (avg_time_per_game * (game_count - result_count) as f32) as u64;
            let hours_remaining = seconds_remaining / 3600;
            let minutes_remaining = (seconds_remaining % 3600) / 60;
            let seconds_remaining = seconds_remaining % 60;
            println!(
                "Estimated time left for generation {} to end: {}h{:02}m{:02}s",
                self.generation, hours_remaining, minutes_remaining, seconds_remaining
            );

            sorted_scores = scores.iter().map(|(_, score)| score.clone()).collect();

            // Sort by score: best scoring first
            sorted_scores.sort_by(|v1, v2| v2.points_per_game().total_cmp(&v1.points_per_game()));

            if let Err(e) = write_report(
                format!("generation_{}", self.generation).as_str(),
                &sorted_scores,
            ) {
                println!("warning: failed to generate report: {}", e)
            };
        }

        let next_gen = next_generation(self.generation, &sorted_scores, self.configs.len());
        self.configs = next_gen;
        self.generation += 1;

        for t in threads {
            t.join().unwrap();
        }
    }
}
