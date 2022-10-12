use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{logic::Direction, protocol::ALL_DIRECTIONS};

pub struct Scorecard {
    up: AtomicUsize,
    down: AtomicUsize,
    right: AtomicUsize,
    left: AtomicUsize,
}

unsafe impl Send for Scorecard {}

impl Scorecard {
    pub fn new() -> Self {
        Self {
            up: AtomicUsize::new(0),
            down: AtomicUsize::new(0),
            right: AtomicUsize::new(0),
            left: AtomicUsize::new(0),
        }
    }

    pub fn get(&self, d: Direction) -> usize {
        match d {
            Direction::Up => self.up.load(Ordering::Relaxed),
            Direction::Down => self.down.load(Ordering::Relaxed),
            Direction::Left => self.left.load(Ordering::Relaxed),
            Direction::Right => self.right.load(Ordering::Relaxed),
        }
    }

    pub fn post_score(&self, d: Direction, score: usize) -> bool {
        score
            <= match d {
                Direction::Up => self.up.fetch_max(score, Ordering::Relaxed),
                Direction::Down => self.down.fetch_max(score, Ordering::Relaxed),
                Direction::Left => self.left.fetch_max(score, Ordering::Relaxed),
                Direction::Right => self.right.fetch_max(score, Ordering::Relaxed),
            }
    }

    pub fn top_score(&self) -> (Direction, usize) {
        let mut top_score = 0;
        let mut best_dir = Direction::Left;
        for dir in ALL_DIRECTIONS {
            let score = self.get(dir);
            if score >= top_score {
                top_score = score;
                best_dir = dir;
            }
        }
        (best_dir, top_score)
    }
}
