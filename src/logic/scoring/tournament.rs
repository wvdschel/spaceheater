use crate::logic::{voronoi, Game};

use super::{kills, turns_survived};

#[derive(Copy, PartialOrd, Ord, Clone, PartialEq, Eq, Default)]
pub struct TournamentScore {
    alive: bool,
    kills: usize,
    tiles: usize,
    turns: usize,
}

impl std::fmt::Display for TournamentScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "alive={}, kills={}, tiles={}, turns={}",
            self.alive, self.kills, self.tiles, self.turns
        ))
    }
}

pub fn tournament(game: &Game) -> TournamentScore {
    let dead = game.you.dead();
    TournamentScore {
        alive: !dead,
        tiles: if !dead { voronoi::me(game) } else { 0 },
        kills: kills(game),
        turns: turns_survived(game),
    }
}
