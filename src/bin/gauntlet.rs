use topsnek::{
    logic::scoring::{self, winter},
    util::gauntlet::Gauntlet,
};

fn main() {
    let mut g = Gauntlet::new(&[
        "--shrinkEveryNTurns",
        "20",
        "-g",
        "wrapped",
        "--map",
        "royale",
        "-t",
        "1000",
    ]);
    g.add_contestant(
        "default wintersnake",
        scoring::winter::Config::<{ u16::MAX }> {
            points_per_food: 30,
            points_per_tile: 10,
            points_per_length_rank: -20,
            points_per_health: 1,
            points_per_distance_to_food: -1,
            points_per_kill: 100,
            points_per_turn_survived: 300,
            points_per_distance_to_smaller_enemies: -1,
            points_when_dead: -1000000,
            hungry_mode_max_health: 35,
            hungry_mode_food_multiplier: 6.0,
            food_distance_cap: 20,
            enemy_distance_cap: 20,
        },
    );
    g.generate_contestants::<winter::Config<{ u16::MAX }>>(50);
    loop {
        g.new_round(1);
    }
}
