use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    thread,
    time::Instant,
};

use crate::logic::Direction;

pub struct Scoretree<S>
where
    S: Ord + Display + Clone + Send,
{
    scores: Mutex<HashMap<Vec<Direction>, S>>,
    send_score: Sender<(Vec<Direction>, S)>,
}

impl<S: Ord + Display + Clone + Send> Scoretree<S> {
    pub fn new(deadline: Instant) -> Self {
        let (tx, rx) = mpsc::channel();
        let scores = Mutex::new(HashMap::new());

        thread::spawn(move || loop {
            match rx.recv_timeout(deadline - Instant::now()) {
                Ok((path, score)) => {
                    let mut scores = scores.lock().unwrap();
                    scores.insert(path, score);
                }
                Err(_) => break,
            }
        });

        Self {
            scores,
            send_score: tx,
        }
    }

    pub fn get_score(&self, path: &Vec<Direction>) -> Option<S> {
        let scores = self.scores.lock().unwrap();

        scores.get(path).map(|s| s.clone())
    }

    pub fn get_scores(&self, paths: &Vec<Vec<Direction>>) -> HashMap<Vec<Direction>, S> {
        let scores = self.scores.lock().unwrap();

        let mut res = HashMap::new();
        for path in paths.iter() {
            if let Some(s) = scores.get(path).map(|s| s.clone()) {
                res.insert(path.clone(), s);
            }
        }
        res
    }

    pub fn get_submission_channel(&self) -> Sender<(Vec<Direction>, S)> {
        self.send_score.clone()
    }
}
