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
mod pairing;
mod report;
mod webserver;

const SCORES: [isize; 4] = [40, -10, -12, -14];

pub trait RandomConfig {
    fn random() -> Self;
}

pub trait GeneticConfig: ToString + Sync + Send {
    fn load(&mut self, cfg: &str);
    fn evolve(&self) -> Box<dyn GeneticConfig>;
    fn battlesnake(&self) -> Box<dyn Battlesnake + Sync + Send>;
    fn boxed_clone(&self) -> Box<dyn GeneticConfig>;
}

pub struct Gauntlet {
    configs: HashMap<String, Box<dyn GeneticConfig>>,
    generation: usize,
    battlesnake_cli_args: Vec<String>,
}

pub struct Score<'a> {
    pub snake_name: String,
    pub snake_config: Option<&'a Box<dyn GeneticConfig>>,
    pub points: isize,
}

impl Gauntlet {
    pub fn new(cli_args: &[&str]) -> Self {
        Self {
            configs: HashMap::new(),
            generation: 0,
            battlesnake_cli_args: cli_args.iter().map(|v| v.to_string()).collect(),
        }
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
                format!("gen{}_snake{}", self.generation, self.configs.len()).as_str(),
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

    pub fn new_round(&mut self, concurrent_games: usize) {
        let all_snake_names = self.all_snake_names();
        let games = pairing::generate_pairings(&all_snake_names, 4);
        let mut scores = HashMap::new();
        for snake in all_snake_names {
            scores.insert(snake, 0 as isize);
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
                *s += SCORES[rank];
                println!(
                    "#{}: {}, points {}, new score is {}",
                    rank + 1,
                    snake,
                    SCORES[rank],
                    *s
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
        }
        println!();
        println!("all scores processed, assembling results");

        let mut scores: Vec<Score> = scores
            .into_iter()
            .map(|(snake, score)| Score {
                snake_config: self.configs.get(&snake),
                snake_name: snake,
                points: score,
            })
            .collect();

        // Sort by score: best scoring first
        scores.sort_by(|v1, v2| v2.points.cmp(&v1.points));

        if let Err(e) = write_report(format!("generation_{}", self.generation).as_str(), &scores) {
            println!("warning: failed to generate report: {}", e)
        };

        let next_gen = next_generation(self.generation, &scores, self.configs.len());
        self.configs = next_gen;
        self.generation += 1;

        for t in threads {
            t.join().unwrap();
        }
    }
}
