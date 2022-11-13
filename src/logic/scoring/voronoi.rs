use crate::logic::{voronoi, Game};

use super::{turns_survived, kills};


#[derive(Copy, Ord, Clone, PartialEq, Eq, Default)]
pub struct VoronoiScore {
    turns_survived: usize,
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
        tiles_controlled: voronoi::me_stack(game),
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
        tiles_controlled: voronoi::me_stack(game),
        kills: kills(game),
        length: game.you.length as isize - max_length,
    }
}

#[derive(Copy, Ord, Clone, PartialEq, Eq, Default)]
pub struct TournamentVoronoiScore {
    survived_by: usize,
    voronoi: VoronoiScore,
}

impl PartialOrd for TournamentVoronoiScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match other.survived_by.partial_cmp(&self.survived_by) {
            // Inverse ordering here
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.voronoi.partial_cmp(&other.voronoi)
    }
}

impl std::fmt::Display for TournamentVoronoiScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "survived_by={}, {}",
            self.survived_by, self.voronoi,
        ))
    }
}

/// tournament_voronoi extends the voronoi_relative_length score function
/// by adding a new, top priority metric: the minimum number of snakes that will outlive
/// our snake. This is used to determine the points allocated in tournament games,
/// so the fewer, the better.
pub fn tournament_voronoi(game: &Game) -> TournamentVoronoiScore {
    let survived_by: usize = if game.you.dead() {
        game.others
            .iter()
            .fold(0, |c, s| if s.dead() { c } else { c + 1 })
    } else {
        0
    };
    TournamentVoronoiScore {
        survived_by,
        voronoi: voronoi(game),
    }
}

pub fn voronoi_me_stack(game: &Game) -> usize {
    voronoi::me_stack(game)
}

pub fn voronoi_me_heap(game: &Game) -> usize {
    voronoi::me_heap(game)
}

pub fn voronoi_all_heap(game: &Game) -> usize {
    let counts = voronoi::all_heap(game);
    *counts.get(&game.you).unwrap_or(&0)
}

pub fn voronoi_all_stack(game: &Game) -> usize {
    let counts = voronoi::all_stack(game);
    *counts.get(&game.you).unwrap_or(&0)
}

pub fn voronoi_all_old(game: &Game) -> usize {
    let counts = voronoi::old(game);
    *counts.get(&game.you).unwrap_or(&0)
}