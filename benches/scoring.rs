use std::{fmt::Display, fs::File};

use criterion::{criterion_group, criterion_main, Criterion};
use topsnek::{
    logic::{self, scoring},
    util::gamelogger,
};

fn load_replay() -> gamelogger::Game {
    gamelogger::Game::load(&mut File::open("./benches/data/game.json.gz").unwrap()).unwrap()
}

fn score_game<S: Ord + Display>(score_fn: fn(&logic::Game) -> S, games: &[logic::Game]) {
    for g in games {
        (score_fn)(g);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let replay = load_replay();
    let games = replay
        .moves
        .iter()
        .map(|(request, _response)| logic::Game::from(request))
        .collect::<Vec<logic::Game>>();

    c.bench_function("classic", |b| {
        b.iter(|| score_game(scoring::classic, games.as_slice()))
    });
    c.bench_function("voronoi", |b| {
        b.iter(|| score_game(scoring::voronoi, games.as_slice()))
    });
    c.bench_function("voronoi_relative_length", |b| {
        b.iter(|| score_game(scoring::voronoi_relative_length, games.as_slice()))
    });
    c.bench_function("tournament_voronoi", |b| {
        b.iter(|| score_game(scoring::tournament_voronoi, games.as_slice()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
