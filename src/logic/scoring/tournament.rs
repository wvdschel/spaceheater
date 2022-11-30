use crate::logic::{voronoi, Game};

use super::{kills, turns_survived};

pub fn tournament(game: &Game) -> i64 {
    let mut score = 0;
    if game.you.dead() {
        score -= 1_000_000_000;
    } else {
        score += voronoi::me(game) as i64 * 1_000;
    }
    score += turns_survived(game) as i64;
    score += kills(game) as i64 * 10_000_000;

    score
}
