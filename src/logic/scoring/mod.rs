use super::Game;

mod voronoi;
pub use voronoi::*;

pub mod pruning;
pub mod tournament;
pub use tournament::tournament as tournament_score;

#[derive(Copy, Ord, Clone, PartialEq, Eq, Default)]
pub struct SurvivalKillsLengthScore {
    turns_survived: usize,
    kills: usize,
    length: usize,
}

impl std::fmt::Display for SurvivalKillsLengthScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "turns={}, kills={}, length={}",
            self.turns_survived, self.kills, self.length
        ))
    }
}

impl PartialOrd for SurvivalKillsLengthScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.turns_survived.partial_cmp(&other.turns_survived) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.kills.partial_cmp(&other.kills) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.length.partial_cmp(&other.length)
    }
}

pub fn turns_survived(game: &Game) -> usize {
    if game.you.health > 0 {
        game.turn
    } else {
        game.turn - 1
    }
}

pub fn kills(game: &Game) -> usize {
    if game.you.health > 0 {
        game.dead_snakes
    } else {
        game.dead_snakes - 1
    }
}

/// classic was my first attempt at a scoring function,
/// optimizing for survival, kills and snake length in that order.
pub fn classic(game: &Game) -> SurvivalKillsLengthScore {
    SurvivalKillsLengthScore {
        turns_survived: turns_survived(game),
        kills: kills(game),
        length: game.you.length,
    }
}
