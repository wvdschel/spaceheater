use std::sync::atomic::{AtomicI64, Ordering};

pub struct AlphaBeta<'a> {
    parent: Option<&'a AlphaBeta<'a>>,
    alpha: AtomicI64,
    beta: AtomicI64,
}

impl<'a> AlphaBeta<'a> {
    pub fn new(a: i64, b: i64) -> Self {
        Self {
            parent: None,
            alpha: AtomicI64::new(a),
            beta: AtomicI64::new(b),
        }
    }

    pub fn new_child(&'a self) -> Self {
        Self {
            parent: Some(self),
            alpha: AtomicI64::new(self.alpha.load(Ordering::Relaxed)),
            beta: AtomicI64::new(self.beta.load(Ordering::Relaxed)),
        }
    }

    #[inline(always)]
    pub fn new_alpha_score(&self, a: i64) -> i64 {
        self.alpha.fetch_max(a, Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn new_beta_score(&self, b: i64) -> i64 {
        self.beta.fetch_min(b, Ordering::Relaxed)
    }

    pub fn should_be_pruned(&self) -> bool {
        let mut max_alpha = self.alpha.load(Ordering::Relaxed);
        let mut min_beta = self.beta.load(Ordering::Relaxed);
        let mut next = self;
        while let Some(v) = next.parent {
            let next_alpha = v.alpha.load(Ordering::Relaxed);
            let next_beta = v.beta.load(Ordering::Relaxed);
            if next_alpha > max_alpha {
                max_alpha = next_alpha
            }
            if next_beta < min_beta {
                min_beta = next_beta
            }
            if max_alpha >= min_beta {
                return true;
            }
            next = v;
        }

        false
    }
}
