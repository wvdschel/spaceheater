use std::{fmt::Display, fs::File, time::Duration};

use pprof::criterion::{Output, PProfProfiler};

use criterion::{criterion_group, criterion_main, Criterion};
use topsnek::{
    logic::{self, voronoi},
    util::gamelogger,
};

fn load_replay_big() -> gamelogger::Game {
    gamelogger::Game::load(&mut File::open("./benches/data/8_players_25x25.json.gz").unwrap())
        .unwrap()
}

fn load_replay() -> gamelogger::Game {
    gamelogger::Game::load(&mut File::open("./benches/data/game.json.gz").unwrap()).unwrap()
}

fn score_games<S: Ord + Display>(score_fn: fn(&logic::Game) -> S, games: &[logic::Game]) {
    // for g in games {
    //     (score_fn)(g);
    // }
    (score_fn)(&games[0]);
}

fn criterion_benchmark(c: &mut Criterion) {
    let replay = load_replay();
    let games = replay
        .moves
        .iter()
        .map(|(request, _response)| logic::Game::from(request))
        .collect::<Vec<logic::Game>>();

    println!("scoring {} games", games.len());

    // c.bench_function("classic", |b| {
    //     b.iter(|| score_game(scoring::classic, games.as_slice()))
    // });

    c.bench_function("voronoi_me", |b| {
        b.iter(|| score_games(voronoi::me, games.as_slice()))
    });

    // c.bench_function("voronoi_all", |b| {
    //     b.iter(|| score_game(scoring::voronoi_all, games.as_slice()))
    // });

    let replay = load_replay_big();
    let games = replay
        .moves
        .iter()
        .map(|(request, _response)| logic::Game::from(request))
        .collect::<Vec<logic::Game>>();

    c.bench_function("voronoi_me_big", |b| {
        b.iter(|| score_games(voronoi::me, games.as_slice()))
    });

    c.bench_function("voronoi_me_big", |b| {
        b.iter(|| score_games(|g| voronoi::me_range_limit(g, 5), games.as_slice()))
    });
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
