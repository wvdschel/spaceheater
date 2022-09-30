use std::fmt::Display;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Snake,
    Head,
    Hazard,
    Food,
    Wall,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Empty => f.write_str("."),
            Tile::Snake => f.write_str("O"),
            Tile::Head => f.write_str("o"),
            Tile::Hazard => f.write_str("!"),
            Tile::Food => f.write_str("*"),
            Tile::Wall => f.write_str("#"),
        }
    }
}

pub trait BoardLike {
    fn get(&self, p: &Point) -> Tile;
    fn set(&mut self, p: &Point, v: Tile);
    fn width(&self) -> isize;
    fn height(&self) -> isize;

    fn is_safe(&self, p: &Point) -> bool {
        match self.get(p) {
            Tile::Empty => true,
            Tile::Food => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for dyn BoardLike {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let p = Point {
                    x,
                    y: self.height() - y - 1,
                };
                _ = self.get(&p).fmt(f);
            }
            _ = f.write_str("\n");
        }
        Ok(())
    }
}

mod board;
pub use board::Board;

pub(crate) mod search;

use crate::protocol::Point;
