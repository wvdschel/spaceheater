use topsnek::{logic::scoring::winter, snakes, util::gauntlet::Gauntlet};

const CHAMPIONS: &str = include_str!("champions.txt");

const SNAKE_COUNT: usize = 30;

fn main() {
    let mut g = Gauntlet::new(&[
        "--shrinkEveryNTurns",
        "20",
        "-g",
        "wrapped",
        "--map",
        "royale",
        "-t",
        "900",
    ]);

    let ref_snake_count = snakes::snakes().len();

    if CHAMPIONS != "" {
        for (i, champ) in CHAMPIONS.split("\n").enumerate() {
            if champ == "" {
                continue;
            }
            let cfg = winter::Config::<{ winter::NumType::MAX }>::try_from(champ).unwrap();
            g.add_contestant(format!("champion_{}", i).as_str(), cfg)
        }
    }
    if g.contestant_count() + ref_snake_count < SNAKE_COUNT {
        g.generate_contestants::<winter::Config<{ winter::NumType::MAX }>>(
            SNAKE_COUNT - g.contestant_count() - ref_snake_count,
        );
    }
    loop {
        g.new_round(3, 4);
    }
}
