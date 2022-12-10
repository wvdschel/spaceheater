use topsnek::{logic::scoring::winter, util::gauntlet::Gauntlet};

const CHAMPIONS: &str = include_str!("champions.txt");

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
    if CHAMPIONS != "" {
        for (i, champ) in CHAMPIONS.split("\n").enumerate() {
            if champ == "" {
                continue;
            }
            let cfg = winter::Config::<{ u16::MAX }>::try_from(champ).unwrap();
            g.add_contestant(format!("champion_{}", i).as_str(), cfg)
        }
    }
    if g.contestant_count() < 75 {
        g.generate_contestants::<winter::Config<{ u16::MAX }>>(75 - g.contestant_count());
    }
    loop {
        g.new_round(4);
    }
}
