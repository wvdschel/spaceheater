use std::sync::RwLock;

pub struct AlphaBeta<'a, S: Ord + Clone> {
    parent: Option<&'a AlphaBeta<'a, S>>,
    alpha: RwLock<Option<S>>,
    beta: RwLock<Option<S>>,
}

impl<'a, S: Ord + Clone> AlphaBeta<'a, S> {
    pub fn new(a: Option<S>, b: Option<S>) -> Self {
        Self {
            parent: None,
            alpha: RwLock::new(a),
            beta: RwLock::new(b),
        }
    }

    pub fn new_child(&'a self) -> Self {
        Self {
            parent: Some(self),
            alpha: RwLock::new(self.alpha.read().unwrap().clone()),
            beta: RwLock::new(self.beta.read().unwrap().clone()),
        }
    }

    pub fn new_alpha_score(&self, a: S) {
        let next_score = Some(a);
        let new_alpha = *self.alpha.read().unwrap() < next_score;

        if new_alpha {
            let mut alpha_write = self.alpha.write().unwrap();
            if *alpha_write < next_score {
                *alpha_write = next_score;
            }
        }
    }

    pub fn new_beta_score(&self, b: S) {
        let new_beta = self
            .beta
            .read()
            .unwrap()
            .as_ref()
            .map_or(true, |old_b| *old_b > b);

        if new_beta {
            let mut beta_write = self.beta.write().unwrap();
            let next_score = Some(b);
            if *beta_write == None || *beta_write > next_score {
                *beta_write = next_score;
            }
        }
    }

    pub fn should_be_pruned(&self) -> bool {
        let mut max_alpha = self.alpha.read().unwrap().clone();
        let mut next = self;
        while let Some(v) = next.parent {
            let other_alpha = v.alpha.read().unwrap();
            if *other_alpha > max_alpha {
                max_alpha = other_alpha.clone();
            }
            next = v;
        }

        let mut next = self;
        loop {
            let beta = next.beta.read().unwrap();
            if *beta != None {
                if max_alpha > *beta {
                    return true;
                }
            }
            if let Some(p) = next.parent {
                next = p;
            } else {
                break;
            }
        }

        false
    }
}
