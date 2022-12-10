use topsnek::{logic::scoring::winter, util::gauntlet::Gauntlet};

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
            let cfg = winter::Config::<{ u16::MAX }>::try_from(champ).unwrap();
            g.add_contestant(format!("champion_{}", i).as_str(), cfg)
        }
    }
    //g.generate_contestants::<winter::Config<{ u16::MAX }>>(50);
    loop {
        g.new_round(4);
    }
}

const CHAMPIONS: &str = include_str!("champions.txt");
