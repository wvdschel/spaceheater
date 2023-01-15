use crate::logic::{floodfill, Game};

use super::{kills, turns_survived};

#[derive(Copy, Ord, Clone, PartialEq, Eq, Default)]
pub struct VoronoiScore {
    turns_survived: i64,
    tiles_controlled: usize,
    kills: usize,
    length: isize,
}

impl PartialOrd for VoronoiScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.turns_survived.partial_cmp(&other.turns_survived) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.tiles_controlled.partial_cmp(&other.tiles_controlled) {
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

impl std::fmt::Display for VoronoiScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "turns={}, tiles={}, kills={}, length={}",
            self.turns_survived, self.tiles_controlled, self.kills, self.length
        ))
    }
}

/// voronoi is a variation of the classic scoring function, using
/// voronoi partitioning to grade games by the number of tiles
/// controlled by our snake.
/// It ranks games by turns survived, tiles controlled, kills and length.
pub fn voronoi(game: &Game) -> VoronoiScore {
    VoronoiScore {
        turns_survived: turns_survived(game),
        tiles_controlled: floodfill::me(game),
        kills: kills(game),
        length: game.you.length as isize,
    }
}

/// voronoi_relative_length is a variation of the voronoi scoring above,
/// but instead of absolute length we optimize for length relative to
/// the longest opponent in the game. This encourages the snake to eat
/// more to keep parity with the enemies, and competes with them for food.
pub fn voronoi_relative_length(game: &Game) -> VoronoiScore {
    let max_length = game
        .others
        .iter()
        .map(|s| s.length)
        .reduce(|max, len| if len > max { len } else { max })
        .unwrap_or(0) as isize;

    VoronoiScore {
        turns_survived: turns_survived(game),
        tiles_controlled: floodfill::me(game),
        kills: kills(game),
        length: game.you.length as isize - max_length,
    }
}

pub fn voronoi_me(game: &Game) -> usize {
    floodfill::me(game)
}

pub fn voronoi_all(game: &Game) -> usize {
    let counts = floodfill::all(game);
    *counts.get(&game.you).unwrap_or(&0)
}
