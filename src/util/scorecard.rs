use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Mutex},
};

use crate::{logic::Direction, protocol::ALL_DIRECTIONS};

pub struct Scorecard {
    scores: Mutex<HashMap<Vec<Direction>, usize>>,
}

unsafe impl Send for Scorecard {}

impl Scorecard {
    pub fn new() -> Self {
        Self {
            scores: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, path: &Vec<Direction>) -> usize {
        let scores = self.scores.lock().unwrap();
        *scores.get(path).unwrap_or(&0)
    }

    pub fn post_score(&self, path: Vec<Direction>, score: usize) -> usize {
        let mut scores = self.scores.lock().unwrap();
        if let Some(&old_score) = scores.get(&path) {
            // This score is a new score for this path.
            // See if this is the worst the opponents can do to use
            // for this path by keeping the lowest scores only.
            if score < old_score {
                scores.insert(path, score);
            }
            old_score
        } else {
            scores.insert(path, score);
            0
        }
    }

    pub fn top_score(&self) -> (Direction, usize) {
        let scores = self.scores.lock().unwrap();
        let mut top_score = 0;
        let mut best_dir = Direction::Left;
        for dir in ALL_DIRECTIONS {
            let &score = scores.get(&vec![dir]).unwrap_or(&0);
            if score >= top_score {
                top_score = score;
                best_dir = dir;
            }
        }
        (best_dir, top_score)
    }
}
