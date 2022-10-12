use std::fmt::Display;

mod board;
pub use board::{Board, BoardOverlay};

mod game;
pub use game::Game;

mod point;
pub use point::Point;

mod snake;
pub use snake::Snake;

pub(crate) mod search;

pub use crate::protocol::Direction;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tile {
    Empty,
    Snake,
    Head,
    Food,
    Hazard,
    HazardWithFood,
    HazardWithSnake,
    HazardWithHead,
    Wall,
}

impl Tile {
    pub fn add(&self, t: Tile) -> Tile {
        match self {
            Tile::Hazard => match t {
                Tile::Snake => Tile::HazardWithSnake,
                Tile::Head => Tile::HazardWithHead,
                Tile::Food => Tile::HazardWithFood,
                _ => t,
            },
            _ => t,
        }
    }

    pub fn clear_snake(&self) -> Tile {
        match self {
            Tile::Snake => Tile::Empty,
            Tile::Head => Tile::Empty,
            Tile::HazardWithSnake => Tile::Hazard,
            Tile::HazardWithHead => Tile::Hazard,
            _ => self.clone(),
        }
    }

    pub fn clear_food(&self) -> Tile {
        match self {
            Tile::Food => Tile::Empty,
            Tile::HazardWithFood => Tile::Empty,
            _ => self.clone(),
        }
    }

    pub fn has_food(&self) -> bool {
        match self {
            Tile::Food => true,
            Tile::HazardWithFood => true,
            _ => false,
        }
    }

    pub fn is_hazard(&self) -> bool {
        match self {
            Tile::Hazard => true,
            Tile::HazardWithFood => true,
            Tile::HazardWithSnake => true,
            Tile::HazardWithHead => true,
            _ => false,
        }
    }

    pub fn is_snake(&self) -> bool {
        match self {
            Tile::Snake => true,
            Tile::Head => true,
            Tile::HazardWithSnake => true,
            Tile::HazardWithHead => true,
            _ => false,
        }
    }

    pub fn is_safe(&self) -> bool {
        match self {
            Tile::Empty => true,
            Tile::Food => true,
            Tile::HazardWithFood => true,
            _ => false,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Empty => f.write_str("."),
            Tile::Snake => f.write_str("O"),
            Tile::Head => f.write_str("o"),
            Tile::Hazard => f.write_str("x"),
            Tile::Food => f.write_str("+"),
            Tile::HazardWithFood => f.write_str("*"),
            Tile::HazardWithSnake => f.write_str("⦻"),
            Tile::HazardWithHead => f.write_str("⦻"),
            Tile::Wall => f.write_str("#"),
        }
    }
}

pub trait BoardLike {
    fn get(&self, p: &Point) -> Tile;
    fn set(&mut self, p: &Point, v: Tile);
    fn width(&self) -> isize;
    fn height(&self) -> isize;

    fn layers(&self) -> usize {
        1
    }

    fn clear_food(&mut self, p: &Point) {
        self.set(p, self.get(p).clear_food())
    }

    fn clear_snake(&mut self, p: &Point) {
        self.set(p, self.get(p).clear_snake())
    }

    fn add(&mut self, p: &Point, v: Tile) {
        self.set(p, self.get(p).add(v))
    }

    fn flatten(&self) -> Board {
        let w = self.width() as usize;
        let h = self.height() as usize;
        let mut res = Board::new(w, h);

        for x in 0..self.width() {
            for y in 0..self.height() {
                let p = Point { x, y };
                res.set(&p, self.get(&p));
            }
        }

        res
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

impl Direction {
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}