use crate::protocol;
pub use protocol::Snake;

use super::{BoardLike, Direction, Tile};

impl Snake {
    pub fn apply_move(
        &mut self,
        dir: Direction,
        board: &mut dyn BoardLike,
        rules: &protocol::Ruleset,
    ) {
        let new_head = if rules.warped_mode() {
            self.head.neighbour(dir).warp(board.width(), board.height())
        } else {
            self.head.neighbour(dir)
        };

        // Apply hazard damage
        if board.get(&new_head).is_hazard() {
            self.health -= rules.settings.hazard_damage_per_turn;
        }

        // Starve snake
        self.health -= 1;

        // Consume food
        if board.get(&new_head).has_food() || rules.constrictor_mode() {
            self.health = 100;
            self.length += 1;
            board.clear_food(&new_head);
        }

        // Apply out of bounds damage
        if new_head.out_of_bounds(board.width(), board.height()) {
            self.health = 0;
        }

        // Update snake position and board
        board.add(&new_head, Tile::Head);
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

    pub fn remove_from_board(&self, board: &mut dyn BoardLike) {
        for t in self.body.iter() {
            board.clear_snake(t)
        }
    }
}

impl std::fmt::Display for Snake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:16} {} (hp={}, len={})",
            self.name,
            std::iter::repeat("o").take(self.length).collect::<String>(),
            self.health,
            self.length
        ))
    }
}
