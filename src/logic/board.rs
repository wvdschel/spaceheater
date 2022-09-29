use crate::protocol::{self, Point};

use super::{Tile, BoardLike};

#[derive(Clone)]
pub struct Board {
    tiles: Vec<Vec<Tile>>,
}

impl Board {
    pub fn new(w: usize, h: usize) -> Board {
        let mut tiles = Vec::with_capacity(w);
        for i in 0..w {
            tiles.push(Vec::with_capacity(h));
            tiles[i].resize(h, Tile::Empty);
        }
        Board { tiles }
    }
}

impl BoardLike for Board {
    fn get(&self, p: &Point) -> Tile {
        if p.x < 0 || p.y < 0 || p.x >= self.width() || p.y >= self.height() {
            return Tile::Wall;
        }
        self.tiles[p.x as usize][p.y as usize]
    }

    fn set(&mut self, p: &Point, v: Tile) {
        if p.x >= 0 && p.y >= 0 && p.x < self.width() && p.y < self.height() {
            self.tiles[p.x as usize][p.y as usize] = v
        }
    }

    fn width(&self) -> isize {
        self.tiles.len() as isize
    }

    fn height(&self) -> isize {
        if self.tiles.len() == 0 {
            0
        } else {
            self.tiles[0].len() as isize
        }
    }
}

impl From<protocol::Board> for Board {
    fn from(g: protocol::Board) -> Self {
        let mut b = Board::new(g.width, g.height);
        for snake in g.snakes {
            for point in snake.body {
                b.set(&point, Tile::Snake)
            }
            b.set(&snake.head, Tile::Head);
        }
        for food in g.food {
            b.set(&food, Tile::Food)
        }
        for hazard in g.hazards {
            b.set(&hazard, Tile::Hazard)
        }
        b
    }
}

pub struct BoardOverlay<'a> {
    tiles: Vec<Vec<Option<Tile>>>,
    below: &'a dyn BoardLike,
}

impl<'a> BoardOverlay<'a> {
    pub fn new(below: &'a dyn BoardLike) -> BoardOverlay {
        let w = below.width() as usize;
        let h = below.height() as usize;
        let mut tiles = Vec::with_capacity(w);
        for i in 0..tiles.len() {
            tiles.push(Vec::with_capacity(h as usize));
            tiles[i].resize(h, None);
        }
        BoardOverlay { tiles, below }
    }
}

impl<'a> BoardLike for BoardOverlay<'a> {
    fn get(&self, p: &Point) -> Tile {
        if p.x < 0 || p.y < 0 || p.x >= self.width() || p.y >= self.height() {
            return Tile::Wall;
        }
        match self.tiles[p.x as usize][p.y as usize] {
            Some(t) => t,
            None => self.below.get(p),
        }
    }

    fn set(&mut self, p: &Point, v: Tile) {
        if p.x >= 0 && p.y >= 0 && p.x < self.width() && p.y < self.height() {
            self.tiles[p.x as usize][p.y as usize] = Some(v)
        }
    }

    fn width(&self) -> isize {
        self.below.width()
    }

    fn height(&self) -> isize {
        self.below.height()
    }
}