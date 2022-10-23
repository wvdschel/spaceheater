use super::{search, Game};

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

pub fn survival_kills_length(game: &Game) -> SurvivalKillsLengthScore {
    SurvivalKillsLengthScore {
        turns_survived: turns_survived(game),
        kills: kills(game),
        length: game.you.length,
    }
}

#[derive(Copy, Ord, Clone, PartialEq, Eq, Default)]
pub struct VoronoiScore {
    turns_survived: usize,
    tiles_controlled: usize,
    kills: usize,
    length: usize,
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

pub fn voronoi(game: &Game) -> VoronoiScore {
    let (_, control_count) = search::voronoi(game);
    VoronoiScore {
        turns_survived: turns_survived(game),
        tiles_controlled: *control_count.get(&game.you.name).unwrap_or(&0),
        kills: kills(game),
        length: game.you.length,
    }
}