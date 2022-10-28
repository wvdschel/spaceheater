use std::collections::VecDeque;

use crate::protocol;

use super::{
    game::{GameMode, Rules},
    Board, Direction, Point, Tile,
};

#[derive(Clone, Hash, Eq)]
pub struct Snake {
    #[cfg(feature = "logging")]
    pub name: String,
    pub id: u8,
    pub health: i8,
    pub body: VecDeque<Point>,
    pub head: Point,
    pub length: usize,
    pub squad: u8,
}

impl PartialEq for Snake {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Snake {
    pub fn apply_move(&mut self, dir: Direction, board: &mut Board, rules: &Rules) {
        let mut new_head = self.head.neighbour(dir);
        if rules.game_mode == GameMode::Wrapped {
            new_head.warp(board.width(), board.height())
        }

        // Apply hazard damage
        if board.get(&new_head).is_hazard() {
            self.health -= rules.hazard_damage_per_turn;
        }

        // Starve snake
        self.health -= 1;

        // Consume food
        if board.get(&new_head).has_food() || rules.game_mode == GameMode::Constrictor {
            self.health = 100;
            self.length += 1;
        }

        // Apply out of bounds damage
        if new_head.out_of_bounds(board.width(), board.height()) {
            self.health = 0;
        }

        // Update snake position and board
        // We intentionally don't add new heads here, as they would break
        // our collision detection in Game.death_by_collission()
        if self.length > 1 {
            board.add(&self.head, Tile::Snake);
        } else {
            board.clear_snake(&self.head);
        }

        self.head = new_head.clone();
        self.body.push_front(new_head);
        if self.body.len() > self.length {
            if let Some(p) = self.body.pop_back() {
                if let Some(p2) = self.body.back() {
                    if &p != p2 {
                        board.clear_snake(&p);
                    }
                } else {
                    board.clear_snake(&p);
                }
            }
        }
    }

    pub fn dead(&self) -> bool {
        self.health <= 0
    }

    pub fn remove_from_board(&self, board: &mut Board) {
        for t in self.body.iter() {
            board.clear_snake(t)
        }
    }
}

impl From<&protocol::Snake> for Snake {
    fn from(s: &protocol::Snake) -> Self {
        Snake {
            #[cfg(feature = "logging")]
            name: s.name,
            id: 0,
            health: s.health as i8,
            body: s.body.clone(),
            head: s.head.clone(),
            length: s.length,
            squad: s.squad.parse().unwrap_or(0),
        }
    }
}

impl std::fmt::Display for Snake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "logging")]
        f.write_fmt(format_args!("{:16} ", self.name))?;
        f.write_fmt(format_args!(
            "{} (hp={}, len={})",
            std::iter::repeat("o").take(self.length).collect::<String>(),
            self.health,
            self.length
        ))
    }
}
