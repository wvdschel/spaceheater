use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Mutex,
};

use crate::{log, logic::Direction, protocol::ALL_DIRECTIONS};

pub struct Scorecard<T: Ord + Default + Copy> {
    scores: Mutex<HashMap<Vec<Direction>, (T, Option<String>)>>,
    paths: Mutex<HashMap<usize, Vec<Vec<Direction>>>>,
    certain_death: Mutex<HashSet<Vec<Direction>>>,
}

unsafe impl<T: Ord + Default + Copy> Send for Scorecard<T> {}

impl<T: Ord + Default + Copy + Display> Scorecard<T> {
    pub fn new() -> Self {
        Self {
            scores: Mutex::new(HashMap::new()),
            paths: Mutex::new(HashMap::new()),
            certain_death: Mutex::new(HashSet::new()),
        }
    }

    #[allow(unused)]
    pub fn get(&self, path: &Vec<Direction>) -> T {
        let scores = self.scores.lock().unwrap();
        scores.get(path).unwrap_or(&(T::default(), None)).0
    }

    pub fn max_step(&self, depth: usize) {
        let mut scores = self.scores.lock().unwrap();
        let all_paths_per_depth = self.paths.lock().unwrap();
        let deaths = self.certain_death.lock().unwrap();

        let mut depth = depth;
        while depth > 0 {
            if let Some(paths) = all_paths_per_depth.get(&depth) {
                for path in paths {
                    // If the path is certain death for a given set of enemy moves, it should not be max-ed.
                    if deaths.contains(path) {
                        continue;
                    }
                    let mut subpath = path.clone();
                    let mut max_score = T::default();
                    let mut max_label = None;
                    if let Some(v) = scores.get(path) {
                        max_score = v.0;
                        max_label = v.1.clone();
                    };
                    for next_move in ALL_DIRECTIONS {
                        subpath.push(next_move);
                        if let Some((score, label)) = &scores.get(&subpath) {
                            if *score > max_score {
                                max_score = *score;
                                max_label = Some(format!(
                                    "max choice: {:?}: {}",
                                    subpath,
                                    label.clone().unwrap_or("".into())
                                ));
                            }
                        }
                        subpath.pop();
                    }
                    scores.insert(subpath, (max_score, max_label));
                }
            }
            depth -= 1
        }
    }

    pub fn post_score(&self, path: Vec<Direction>, score: T, label: Option<String>) -> T {
        let mut scores = self.scores.lock().unwrap();
        let old_score = scores.get(&path).map(|(s, _)| *s);
        if let Some(old_score) = old_score {
            // This score is a new score for an existing path.
            // See if this is the worst the opponents can do to us
            // for this path by keeping the lowest scores only.
            if score < old_score {
                scores.insert(path, (score, label));
            }
            old_score
        } else {
            let mut paths = self.paths.lock().unwrap();

            if let Some(paths) = paths.get_mut(&path.len()) {
                paths.push(path.clone());
            } else {
                paths.insert(path.len(), vec![path.clone()]);
            }
            scores.insert(path, (score, label));
            T::default()
        }
    }

    pub fn post_certain_death(&self, path: Vec<Direction>) {
        let mut deaths = self.certain_death.lock().unwrap();
        #[cfg(feature = "logging")]
        if !deaths.contains(&path) {
            log!("Marking as certain death: {:?}", path);
        }
        deaths.insert(path);
    }

    pub fn is_certain_death(&self, path: &Vec<Direction>) -> bool {
        let deaths = self.certain_death.lock().unwrap();
        deaths.contains(path)
    }

    pub fn top_score(&self) -> (Direction, T) {
        let scores = self.scores.lock().unwrap();
        let mut top_score = T::default();
        let mut best_dir = Direction::Left;
        for dir in ALL_DIRECTIONS {
            let score = scores
                .get(&vec![dir])
                .map(|(s, _)| *s)
                .unwrap_or(T::default());
            if score >= top_score {
                top_score = score;
                best_dir = dir;
            }
        }
        (best_dir, top_score)
    }
}

impl<T: Ord + Default + Copy + Display> std::fmt::Display for Scorecard<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scores = self.scores.lock().unwrap();
        let paths = self.paths.lock().unwrap();
        for depth in paths.keys().sorted() {
            for path in &paths[depth] {
                let (score, label) = scores.get(path).unwrap();
                let label = match label {
                    Some(v) => v.clone(),
                    None => String::new(),
                };
                f.write_fmt(format_args!("{:?}: score {} - {}\n", path, score, label))?
            }
        }
        Ok(())
    }
}
