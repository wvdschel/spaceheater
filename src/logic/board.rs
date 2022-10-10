use std::sync::Arc;

use crate::protocol::{self, Point};

use super::{BoardLike, Tile};

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

impl From<&protocol::Board> for Board {
    fn from(g: &protocol::Board) -> Self {
        let mut b = Board::new(g.width, g.height);
        for snake in &g.snakes {
            for point in &snake.body {
                b.add(&point, Tile::Snake)
            }
            b.add(&snake.head, Tile::Head);
        }
        for food in &g.food {
            b.add(&food, Tile::Food)
        }
        for hazard in &g.hazards {
            b.add(&hazard, Tile::Hazard)
        }
        b
    }
}

pub struct BoardOverlay {
    tiles: Vec<Vec<Option<Tile>>>,
    below: Arc<dyn BoardLike>,
}

impl BoardOverlay {
    pub fn new(below: Arc<dyn BoardLike>) -> BoardOverlay {
        if below.layers() as isize > below.width() / 2 {
            return Self::new(Arc::new(below.flatten()));
        }

        let w = below.width() as usize;
        let h = below.height() as usize;
        let mut tiles = Vec::with_capacity(w);
        for x in 0..w {
            tiles.push(Vec::with_capacity(h as usize));
            tiles[x].resize(h, None);
        }
        BoardOverlay { tiles, below }
    }
}

impl BoardLike for BoardOverlay {
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

    fn layers(&self) -> usize {
        self.below.layers() + 1
    }
}
