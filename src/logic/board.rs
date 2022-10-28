use crate::{
    log,
    protocol::{self, Point},
};

use super::Tile;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Board {
    pub(super) data: Vec<u8>,
}

const TILE_MASK: u8 = 0b1111;
const HAZARD_MASK: u8 = 0b1100;
const TILE_TYPE_MASK: u8 = 0b0011;
const EMPTY: u8 = 0b00;
const SNAKE: u8 = 0b01;
const HEAD: u8 = 0b10;
const FOOD: u8 = 0b11;

macro_rules! get_tile {
    ($data:expr, $x:expr, $y:expr, $mask:expr) => {{
        let w = $data[0] as usize;
        let (x, y) = ($x as usize, $y as usize);
        let idx = x + y * w;
        if idx % 2 == 0 {
            (unsafe { $data.get_unchecked(2 + idx / 2) } >> 4) & $mask
        } else {
            (unsafe { $data.get_unchecked(2 + idx / 2) }) & $mask
        }
    }};
}

macro_rules! set_tile {
    ($data:expr, $x:expr, $y:expr, $mask:expr, $value:expr) => {{
        let w = $data[0] as usize;
        let (x, y) = ($x as usize, $y as usize);
        let idx = x + y * w;
        let v = unsafe { $data.get_unchecked_mut(2 + idx / 2) };
        if idx % 2 == 0 {
            let mask = $mask << 4;
            let value = $value << 4;
            *v = (!mask & *v | mask & value);
        } else {
            *v = (!$mask & *v | $mask & $value);
        }
    }};
}

impl Board {
    pub fn new(w: usize, h: usize) -> Board {
        let mut count = w * h + 2;
        if count % 2 == 1 {
            count += 1
        }
        let mut data = Vec::with_capacity(count);
        data.resize(count, 0 as u8);
        data[0] = w as u8;
        data[1] = h as u8;
        Board { data }
    }

    #[inline(always)]
    fn check_type(&self, p: &Point, mask: u8) -> bool {
        if p.x < 0 || p.y < 0 || p.x >= self.data[0] as i8 || p.y >= self.data[1] as i8 {
            return false;
        }
        get_tile!(self.data, p.x, p.y, mask) == mask
    }

    #[inline(always)]
    pub fn is_snake(&self, p: &Point) -> bool {
        self.check_type(p, SNAKE)
    }

    #[inline(always)]
    pub fn is_head(&self, p: &Point) -> bool {
        self.check_type(p, HEAD)
    }

    #[inline(always)]
    pub fn is_food(&self, p: &Point) -> bool {
        self.check_type(p, FOOD)
    }

    #[inline(always)]
    pub fn is_empty(&self, p: &Point) -> bool {
        self.check_type(p, EMPTY)
    }

    #[inline(always)]
    pub fn hazard_count(&self, p: &Point) -> u8 {
        if p.x < 0 || p.y < 0 || p.x as isize >= self.width() || p.y as isize >= self.height() {
            return 0;
        }
        get_tile!(self.data, p.x, p.y, HAZARD_MASK) >> 2
    }

    pub fn set(&mut self, p: &Point, t: Tile) {
        if p.x < 0 || p.y < 0 || p.x as isize >= self.width() || p.y as isize >= self.height() {
            return;
        }
        match t {
            Tile::Empty => set_tile!(self.data, p.x, p.y, TILE_MASK, EMPTY),
            Tile::Snake => set_tile!(self.data, p.x, p.y, TILE_MASK, SNAKE),
            Tile::Head => set_tile!(self.data, p.x, p.y, TILE_MASK, HEAD),
            Tile::Food => set_tile!(self.data, p.x, p.y, TILE_MASK, FOOD),
            Tile::Hazard(x) => set_tile!(self.data, p.x, p.y, TILE_MASK, (x as u8 & 3) << 2),
            Tile::HazardWithFood(x) => {
                set_tile!(self.data, p.x, p.y, TILE_MASK, ((x as u8 & 3) << 2) | FOOD)
            }
            Tile::HazardWithSnake(x) => {
                set_tile!(self.data, p.x, p.y, TILE_MASK, ((x as u8 & 3) << 2) | SNAKE)
            }
            Tile::HazardWithHead(x) => {
                set_tile!(self.data, p.x, p.y, TILE_MASK, ((x as u8 & 3) << 2) | HEAD)
            }
            Tile::Wall => {
                log!("warning: attempt to configure a tile as a wall, this should never happen. Adding max hazards instead.");
                set_tile!(self.data, p.x, p.y, TILE_MASK, 3 << 2);
            }
        }
    }

    #[inline(always)]
    pub fn damage(&self, p: &Point, hazard_dmg: i8) -> i8 {
        let t = get_tile!(self.data, p.x, p.y, TILE_MASK);
        match t & TILE_TYPE_MASK {
            EMPTY => {
                let hazards = (t & HAZARD_MASK) as u8 >> 2;
                hazard_dmg * hazards as i8
            }
            FOOD => 0,
            SNAKE | HEAD => i8::MAX,
            _ => unreachable!(),
        }
    }

    pub fn get(&self, p: &Point) -> Tile {
        if p.x < 0 || p.y < 0 || p.x as isize >= self.width() || p.y as isize >= self.height() {
            return Tile::Wall;
        }
        let value = get_tile!(self.data, p.x, p.y, TILE_MASK);
        let hazards = (value & HAZARD_MASK) as u8 >> 2;
        if hazards > 0 {
            match value & TILE_TYPE_MASK {
                EMPTY => Tile::Hazard(hazards),
                HEAD => Tile::HazardWithHead(hazards),
                SNAKE => Tile::HazardWithSnake(hazards),
                FOOD => Tile::HazardWithFood(hazards),
                _ => panic!(),
            }
        } else {
            match value & TILE_TYPE_MASK {
                EMPTY => Tile::Empty,
                HEAD => Tile::Head,
                SNAKE => Tile::Snake,
                FOOD => Tile::Food,
                _ => panic!(),
            }
        }
    }
    pub fn add(&mut self, p: &Point, t: Tile) {
        match t {
            Tile::Empty => set_tile!(self.data, p.x, p.y, TILE_TYPE_MASK, EMPTY),
            Tile::Snake => set_tile!(self.data, p.x, p.y, TILE_TYPE_MASK, SNAKE),
            Tile::Head => set_tile!(self.data, p.x, p.y, TILE_TYPE_MASK, HEAD),
            Tile::Food => set_tile!(self.data, p.x, p.y, TILE_TYPE_MASK, FOOD),
            Tile::Hazard(x) => {
                let mut hazard_count = self.hazard_count(p) + x as u8;
                if hazard_count > 3 {
                    hazard_count = 3;
                }
                set_tile!(self.data, p.x, p.y, HAZARD_MASK, hazard_count << 2);
            }
            Tile::HazardWithFood(x) => {
                let mut hazard_count = self.hazard_count(p) + x as u8;
                if hazard_count > 3 {
                    hazard_count = 3;
                }
                set_tile!(self.data, p.x, p.y, TILE_MASK, (hazard_count << 2) | FOOD)
            }
            Tile::HazardWithSnake(x) => {
                let mut hazard_count = self.hazard_count(p) + x as u8;
                if hazard_count > 3 {
                    hazard_count = 3;
                }
                set_tile!(self.data, p.x, p.y, TILE_MASK, (hazard_count << 2) | SNAKE)
            }
            Tile::HazardWithHead(x) => {
                let mut hazard_count = self.hazard_count(p) + x as u8;
                if hazard_count > 3 {
                    hazard_count = 3;
                }
                set_tile!(self.data, p.x, p.y, TILE_MASK, (hazard_count << 2) | HEAD)
            }
            Tile::Wall => {
                println!("warning: attempt to configure a tile as a wall, this should never happen. Adding max hazards instead.");
                set_tile!(self.data, p.x, p.y, HAZARD_MASK, 3 << 2);
            }
        }
    }

    pub fn clear_snake(&mut self, p: &Point) {
        self.add(p, Tile::Empty);
    }

    #[inline(always)]
    pub fn width(&self) -> isize {
        self.data[0] as isize
    }

    #[inline(always)]
    pub fn height(&self) -> isize {
        self.data[1] as isize
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
            b.add(&hazard, Tile::Hazard(1))
        }
        b
    }
}

impl ToString for Board {
    fn to_string(&self) -> String {
        let mut parts = vec![];
        for y in 0..self.height() {
            let y = self.height() - 1 - y;
            for x in 0..self.width() {
                parts.push(format!(
                    "{}",
                    self.get(&Point {
                        x: x as i8,
                        y: y as i8,
                    })
                ))
            }
            parts.push("\n".to_string());
        }
        parts.join("")
    }
}
