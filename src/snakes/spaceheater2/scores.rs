use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    thread,
};

use crate::logic::Direction;

pub struct Scoretree<S>
where
    S: Ord + Display + Clone,
{
    scores: Mutex<HashMap<Vec<Direction>, S>>,
    recv_score: Receiver<(Vec<Direction>, S)>,
    send_score: Sender<(Vec<Direction>, S)>,
}

impl<S: Ord + Display + Clone> Scoretree<S> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            scores: Mutex::new(HashMap::new()),
            recv_score: rx,
            send_score: tx,
        }
    }

    pub fn get_score(&self, path: &Vec<Direction>) -> Option<S> {
        let scores = self.scores.lock().unwrap();

        scores.get(path).map(|s| s.clone())
    }

    pub fn get_scores(&self, paths: &Vec<Vec<Direction>>) -> HashMap<Vec<Direction>, S> {
        todo!()
    }

    pub fn get_submission_channel(&self) -> Sender<(Vec<Direction>, S)> {
        self.send_score.clone()
    }
}
