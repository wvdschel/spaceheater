use std::{
    fs::File,
    time::{Duration, Instant},
};

use pprof::criterion::{Output, PProfProfiler};

use criterion::{criterion_group, criterion_main, Criterion};
use topsnek::{
    logic::{self, scoring},
    snakes::Spaceheater3,
    util::gamelogger,
};

fn load_replay() -> gamelogger::Game {
    gamelogger::Game::load(
        &mut File::open("./sample_games/4_players_11x11_wrapped_royale.json.gz").unwrap(),
    )
    .unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let replay = load_replay();
    let games = replay
        .moves
        .iter()
        .map(|(request, _response)| logic::Game::from(request))
        .collect::<Vec<logic::Game>>();

    let spaceheater = Spaceheater3::new(scoring::tournament_score, None);
    let deadline = Instant::now() + Duration::from_secs(10000);

    let count = games.len();
    if count < 5 {
        panic!("need at least 5 turns")
    }

    for depth in [4, 5, 6, 7, 8] {
        for turn in [count / 5, 2 * count / 5, 3 * count / 5, 4 * count / 5] {
            for mult in [0.0, 1.0, 2.0, 3.0] {
                c.bench_function(
                    format!("turn_{}_depth_{}_thread_mult_{}", turn, depth, mult).as_str(),
                    |b| b.iter(|| spaceheater.solve(games[turn].clone(), &deadline, depth, mult)),
                );
            }
        }
    }
}

criterion_group! {
  name = spaceheater_forking;
  config = Criterion::default().
    with_profiler(PProfProfiler::new(1000, Output::Flamegraph(None))).
    measurement_time(Duration::from_secs(20)).
    sample_size(10);
  targets = criterion_benchmark
}

criterion_main!(spaceheater_forking);
