use topsnek::{logic::scoring::winter, snakes, util::gauntlet::Gauntlet};

const CHAMPIONS: &str = include_str!("champions.txt");

const SNAKE_COUNT: usize = 60;

fn main() {
    let mut g = Gauntlet::new(&[
        "--shrinkEveryNTurns",
        "20",
        "-g",
        "wrapped",
        "--map",
        "royale",
        "-t",
        "1100",
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
        let gen_count = SNAKE_COUNT - g.contestant_count() - ref_snake_count;
        println!("Generating {} random snakes", gen_count);
        g.generate_contestants::<winter::Config<{ winter::NumType::MAX }>>(gen_count);
    }
    loop {
        g.new_round(1, 4);
    }
}
