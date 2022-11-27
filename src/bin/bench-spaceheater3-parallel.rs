use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    time::{Duration, Instant},
};

use topsnek::{
    logic::{scoring, Game},
    snakes::spaceheater3::{max::MaximizingNode, parallel::AlphaBeta},
    util::{gamelogger, thread_count},
};

fn load_all_moves_by_snake_count() -> HashMap<usize, Vec<Game>> {
    let mut res = HashMap::new();

    for path in fs::read_dir("./logs").unwrap() {
        let path = path.unwrap().path();
        if path.is_file() && path.to_str().unwrap_or("").ends_with(".json.gz") {
            let mut f = File::open(path).unwrap();
            let game = gamelogger::Game::load(&mut f).unwrap();
            for turn in game.moves {
                let game = Game::from(&turn.0);
                let snake_count = game.others.len();
                if !res.contains_key(&snake_count) {
                    res.insert(snake_count, vec![game]);
                } else {
                    res.get_mut(&snake_count).unwrap().push(game);
                }
            }
        }
    }

    res
}

macro_rules! generate_datafile {
    // `()` indicates that the macro takes no argument.
    ($games:expr, $enemies:expr, $base_depth:expr, $leaves:literal) => {
        let mut file = File::create(format!("measurements/{:02}_enemies_{}.dat", $enemies, $leaves)).unwrap();
        print!("{:10}: ", $leaves);
        for (idx, g) in $games.iter().enumerate() {
            let mut root = MaximizingNode::<scoring::tournament::TournamentScore>::new(g.clone());

            let mut best_score = None;
            for max_depth in ($base_depth + 1)..usize::MAX {
                let (res, _) = root.par_solve::<_, 100>(
                    &(Instant::now() + TIMEOUT),
                    max_depth,
                    &scoring::tournament::tournament,
                    &AlphaBeta::new(None, None),
                    thread_count() as f32,
                );
                let curr_score = res.as_ref().map(|s| s.1);
                if curr_score == None || best_score >= curr_score {
                    file.write_all(format!("{} {}\n", idx, max_depth).as_bytes())
                        .unwrap();
                    break;
                }
                best_score = curr_score;
            }
            print!(".");
            io::stdout().flush().unwrap();
        }
        println!();

    };
    ($games:expr, $enemies:expr, $base_depth:expr, $leaves:literal, $($other_leaves:literal),+) => {
        generate_datafile!($games, $enemies, $base_depth, $leaves);
        generate_datafile!($games, $enemies, $base_depth, $($other_leaves),+);
    };
}

const TIMEOUT: Duration = Duration::from_millis(500);
const MAX_GAMES: usize = 500;

fn main() {
    let all_moves = load_all_moves_by_snake_count();

    for (snake_count, games) in all_moves.iter() {
        println!("turns with {} other snakes: {}", snake_count, games.len());

        let base_depth = match snake_count {
            0 => 5,
            1 => 3,
            2 => 2,
            3 => 2,
            4 => 2,
            _ => 1,
        };

        let games = if games.len() > MAX_GAMES {
            println!("limiting to {} games per enemy count", MAX_GAMES);
            let mut res = Vec::with_capacity(MAX_GAMES);
            for g in &games[0..MAX_GAMES] {
                res.push(g.clone());
            }
            res
        } else {
            games.clone()
        };

        let mut file =
            File::create(format!("measurements/{:02}_enemies_base.dat", snake_count)).unwrap();
        print!("{:10}: ", "base");
        for (idx, g) in games.iter().enumerate() {
            let mut root = MaximizingNode::<scoring::tournament::TournamentScore>::new(g.clone());

            let mut best_score = None;
            for max_depth in (base_depth + 1)..usize::MAX {
                let (res, _) = root.solve(
                    &(Instant::now() + TIMEOUT),
                    max_depth,
                    &scoring::tournament::tournament,
                    None,
                    None,
                );
                let curr_score = res.as_ref().map(|s| s.1);
                if curr_score == None || best_score >= curr_score {
                    file.write_all(format!("{} {}\n", idx, max_depth).as_bytes())
                        .unwrap();
                    break;
                }
                best_score = curr_score;
            }
            print!(".");
            io::stdout().flush().unwrap();
        }
        println!();

        generate_datafile!(
            games,
            snake_count,
            base_depth,
            2_000,
            8_000,
            20_000,
            50_000,
            100_000,
            200_000,
            500_000
        );

        let mut plot_file =
            File::create(format!("measurements/{:02}_enemies.gp", snake_count)).unwrap();
        plot_file
            .write_all(
                format!(
                    "set terminal png size 1920,1080\nset output '{:02}_enemies.png'\n",
                    snake_count
                )
                .as_bytes(),
            )
            .unwrap();
        plot_file
            .write_all(
                format!("plot '{:02}_enemies_base.dat' title 'base'", snake_count).as_bytes(),
            )
            .unwrap();
        for leaves in [2_000, 8_000, 20_000, 50_000, 100_000, 200_000, 500_000] {
            plot_file
                .write_all(
                    format!(
                        ", '{:02}_enemies_{}.dat' title 'leaves={}'",
                        snake_count, leaves, leaves,
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    }
}
