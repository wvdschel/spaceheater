use std::fs;
use topsnek::logic::{self, scoring};
use topsnek::util::gamelogger;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn load_replays() -> Vec<gamelogger::Game> {
    let mut res = vec![];

    for replay in fs::read_dir("./benches/gamedata").unwrap() {
        let replay = replay.unwrap();
        let mut file = fs::File::open(replay.path()).unwrap();
        let game = gamelogger::Game::load(&mut file).unwrap();
        res.push(game);
    }

    res
}

fn criterion_benchmark(c: &mut Criterion) {
    let replays = load_replays();
    let game = logic::Game::from(&replays[0].start_request);

    c.bench_function("classic", |b| b.iter(|| scoring::classic(black_box(&game))));
    c.bench_function("voronoi", |b| b.iter(|| scoring::voronoi(black_box(&game))));
    c.bench_function("voronoi_relative_length", |b| {
        b.iter(|| scoring::voronoi_relative_length(black_box(&game)))
    });
    c.bench_function("tournament_voronoi", |b| {
        b.iter(|| scoring::tournament_voronoi(black_box(&game)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
