pub use crate::protocol::Point;

use super::Direction;

impl Point {
    pub fn neighbour(&self, d: Direction) -> Point {
        match d {
            Direction::Up => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }

    pub fn neighbours(&self) -> [(Direction, Point); 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .map(|d| (d, self.neighbour(d)))
    }

    pub fn out_of_bounds(&self, width: isize, height: isize) -> bool {
        let (width, height) = (width as i8, height as i8);
        self.x < 0 || self.y < 0 || self.x >= width || self.y >= height
    }

    pub fn warp(&self, width: isize, height: isize) -> Point {
        let (width, height) = (width as i8, height as i8);
        let mut res = self.clone();
        if self.x == -1 {
            res.x = width - 1;
        }
        if self.x == width {
            res.x = 0;
        }
        if self.y == -1 {
            res.y = height - 1;
        }
        if self.y == height {
            res.y = 0;
        }
        res
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({},{})", self.x, self.y))
    }
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
