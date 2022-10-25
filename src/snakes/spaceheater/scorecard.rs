use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Mutex,
};

#[allow(unused)]
use crate::{log, logic::Direction, protocol::ALL_DIRECTIONS};

struct ScorecardInternal<T: Ord + Default + Copy> {
    scores: HashMap<Vec<Direction>, (T, Option<String>)>,
    paths: HashMap<usize, Vec<Vec<Direction>>>,
    certain_death: HashSet<Vec<Direction>>,
}

impl<T: Ord + Default + Copy + Display> ScorecardInternal<T> {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            paths: HashMap::new(),
            certain_death: HashSet::new(),
        }
    }
}

pub struct Scorecard<T: Ord + Default + Copy> {
    state: Mutex<ScorecardInternal<T>>,
}

unsafe impl<T: Ord + Default + Copy> Send for Scorecard<T> {}

impl<T: Ord + Default + Copy + Display> Scorecard<T> {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ScorecardInternal::new()),
        }
    }

    #[allow(unused)]
    pub fn get(&self, path: &Vec<Direction>) -> T {
        let state = self.state.lock().unwrap();
        state.scores.get(path).unwrap_or(&(T::default(), None)).0
    }

    pub fn max_step(&self, depth: usize) {
        let mut state = self.state.lock().unwrap();
        let all_paths_per_depth = state.paths.clone();

        let mut depth = depth;
        while depth > 0 {
            if let Some(paths) = all_paths_per_depth.get(&depth) {
                for path in paths {
                    // If the path is certain death for a given set of enemy moves, it should not be max-ed.
                    if state.certain_death.contains(path) {
                        continue;
                    }
                    let mut subpath = path.clone();
                    let mut max_score = T::default();
                    let mut max_label = None;
                    if let Some(v) = state.scores.get(path) {
                        max_score = v.0;
                        max_label = v.1.clone();
                    };
                    for next_move in ALL_DIRECTIONS {
                        subpath.push(next_move);
                        if let Some((score, label)) = &state.scores.get(&subpath) {
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
                    state.scores.insert(subpath, (max_score, max_label));
                }
            }
            depth -= 1
        }
    }

    pub fn post_score(&self, path: Vec<Direction>, score: T, label: Option<String>) -> T {
        let mut state = self.state.lock().unwrap();
        let old_score = state.scores.get(&path).map(|(s, _)| *s);
        if let Some(old_score) = old_score {
            // This score is a new score for an existing path.
            // See if this is the worst the opponents can do to us
            // for this path by keeping the lowest scores only.
            if score < old_score {
                state.scores.insert(path, (score, label));
            }
            old_score
        } else {
            if let Some(paths) = state.paths.get_mut(&path.len()) {
                paths.push(path.clone());
            } else {
                state.paths.insert(path.len(), vec![path.clone()]);
            }
            state.scores.insert(path, (score, label));
            T::default()
        }
    }

    pub fn post_certain_death(&self, path: Vec<Direction>) {
        let mut state = self.state.lock().unwrap();
        #[cfg(feature = "logging")]
        if !state.certain_death.contains(&path) {
            log!("Marking as certain death: {:?}", path);
        }
        state.certain_death.insert(path);
    }

    pub fn is_certain_death(&self, path: &Vec<Direction>) -> bool {
        let state = self.state.lock().unwrap();
        state.certain_death.contains(path)
    }

    pub fn top_score(&self) -> (Direction, T) {
        let state = self.state.lock().unwrap();

        let mut top_score = T::default();
        let mut best_dir = Direction::Left;
        for dir in ALL_DIRECTIONS {
            let score = state
                .scores
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
        let state = self.state.lock().unwrap();
        for depth in state.paths.keys().sorted() {
            for path in &state.paths[depth] {
                let (score, label) = state.scores.get(path).unwrap();
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
