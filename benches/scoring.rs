use std::{fs::File, time::Duration};

use pprof::criterion::{Output, PProfProfiler};

use criterion::{criterion_group, criterion_main, Criterion};
use topsnek::{
    logic::{
        self,
        scoring::{
            self,
            winter::{self, Config},
        },
        voronoi,
    },
    util::gamelogger,
};

#[allow(unused)]
fn load_replay_big() -> gamelogger::Game {
    gamelogger::Game::load(&mut File::open("./benches/data/8_players_25x25.json.gz").unwrap())
        .unwrap()
}

fn load_replay() -> gamelogger::Game {
    gamelogger::Game::load(&mut File::open("./benches/data/game.json.gz").unwrap()).unwrap()
}

fn score_game<S, F>(score_fn: F, game: &logic::Game)
where
    F: Fn(&logic::Game) -> S,
{
    (score_fn)(game);
}

fn criterion_benchmark(c: &mut Criterion) {
    let replay = load_replay();
    let games = replay
        .moves
        .iter()
        .map(|(request, _response)| logic::Game::from(request))
        .collect::<Vec<logic::Game>>();

    let count = games.len();
    if count < 8 {
        panic!("need at least 8 turns")
    }

    let turns: Vec<usize> = (0..8).map(|i| i * count / 8).collect();

    for turn in turns {
        c.bench_function(format!("classic_turn_{}", turn).as_str(), |b| {
            b.iter(|| score_game(scoring::classic, &games[turn]))
        });

        c.bench_function(format!("voronoi_me_turn_{}", turn).as_str(), |b| {
            b.iter(|| score_game(voronoi::me, &games[turn]))
        });

        c.bench_function(format!("winter_turn_{}", turn).as_str(), |b| {
            b.iter(|| {
                score_game(
                    winter::winter_score::<11, 11, 4, { u16::MAX }>(Config {
                        points_per_food: 30,
                        points_per_tile: 10,
                        points_per_length_rank: -20,
                        points_per_health: 1,
                        points_per_distance_to_food: -1,
                        points_per_kill: 100,
                        points_per_turn_survived: 300,
                        points_per_distance_to_smaller_enemies: -1,
                        points_when_dead: -1000000,
                        hungry_mode_max_health: 35,
                        hungry_mode_food_multiplier: 6.0,
                    }),
                    &games[turn],
                )
            })
        });

        // c.bench_function(format!("voronoi_all_turn_{}", turn).as_str(), |b| {
        //     b.iter(|| score_game(scoring::voronoi_all, &games[turn]))
        // });

        // c.bench_function(format!("voronoi_limit5_turn_{}", turn).as_str(), |b| {
        //     b.iter(|| score_game(|g| voronoi::me_range_limit(g, 5), &games[turn]))
        // });

        // c.bench_function(format!("voronoi_limit8_turn_{}", turn).as_str(), |b| {
        //     b.iter(|| score_game(|g| voronoi::me_range_limit(g, 8), &games[turn]))
        // });
    }
}

criterion_group! {
  name = scoring;
  config = Criterion::default().
    with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None))).
    measurement_time(Duration::from_secs(7)).
    sample_size(25);
  targets = criterion_benchmark
}

criterion_main!(scoring);
