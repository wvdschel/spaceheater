use std::{collections::HashMap, sync::Mutex};

use crate::{logic::Direction, protocol::ALL_DIRECTIONS};

pub struct Scorecard<T: Ord + Default + Copy> {
    scores: Mutex<HashMap<Vec<Direction>, T>>,
    paths: Mutex<HashMap<usize, Vec<Vec<Direction>>>>,
}

unsafe impl<T: Ord + Default + Copy> Send for Scorecard<T> {}

impl<T: Ord + Default + Copy> Scorecard<T> {
    pub fn new() -> Self {
        Self {
            scores: Mutex::new(HashMap::new()),
            paths: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, path: &Vec<Direction>) -> T {
        let scores = self.scores.lock().unwrap();
        *scores.get(path).unwrap_or(&T::default())
    }

    pub fn max_step(&self, depth: usize) {
        let mut scores = self.scores.lock().unwrap();
        let paths = self.paths.lock().unwrap();

        let mut depth = depth;
        while depth > 0 {
            if let Some(paths) = paths.get(&depth) {
                for path in paths {
                    let mut subpath = path.clone();
                    let mut max_score = T::default();
                    for next_move in ALL_DIRECTIONS {
                        subpath.push(next_move);
                        if let Some(&score) = scores.get(&subpath) {
                            if score > max_score {
                                max_score = score
                            }
                        }
                        subpath.pop();
                    }
                    scores.insert(subpath, max_score);
                }
            }
            depth -= 1
        }
    }

    pub fn post_score(&self, path: Vec<Direction>, score: T) -> T {
        let mut scores = self.scores.lock().unwrap();
        if let Some(&old_score) = scores.get(&path) {
            // This score is a new score for an existing path.
            // See if this is the worst the opponents can do to us
            // for this path by keeping the lowest scores only.
            if score < old_score {
                scores.insert(path, score);
            }
            old_score
        } else {
            let mut paths = self.paths.lock().unwrap();

            if let Some(paths) = paths.get_mut(&path.len()) {
                paths.push(path.clone());
            } else {
                paths.insert(path.len(), vec![path.clone()]);
            }
            scores.insert(path, score);
            T::default()
        }
    }

    pub fn top_score(&self) -> (Direction, T) {
        let scores = self.scores.lock().unwrap();
        let mut top_score = T::default();
        let mut best_dir = Direction::Left;
        for dir in ALL_DIRECTIONS {
            let &score = scores.get(&vec![dir]).unwrap_or(&T::default());
            if score >= top_score {
                top_score = score;
                best_dir = dir;
            }
        }
        (best_dir, top_score)
    }
}
