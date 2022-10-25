use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use topsnek::{
    logic::{self, scoring},
    snakes,
    util::gamelogger,
};

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

fn solve_game(game: &logic::Game, max_depth: usize) {
    let mut solver = snakes::spaceheater::GameSolver::new(scoring::tournament_voronoi);
    solver.solve("", black_box(&game), None, max_depth);
}

fn criterion_benchmark(c: &mut Criterion) {
    let replays = load_replays();
    let game = logic::Game::from(&replays[0].start_request);

    c.bench_function("spaceheater depth 1", |b| b.iter(|| solve_game(&game, 1)));
    c.bench_function("spaceheater depth 2", |b| b.iter(|| solve_game(&game, 2)));
    c.bench_function("spaceheater depth 3", |b| b.iter(|| solve_game(&game, 3)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
