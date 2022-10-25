use std::{
    collections::{hash_map::DefaultHasher, HashSet, VecDeque},
    hash::Hasher,
    sync::{Condvar, Mutex},
};

use std::hash::Hash;

struct WorkList<W: Hash + Eq> {
    work_items: VecDeque<W>,
    items_seen: HashSet<u64>,
    work_count: usize,
    duplicate_count: usize,
}

impl<W: Hash + Eq> WorkList<W> {
    pub fn new() -> Self {
        Self {
            work_items: VecDeque::new(),
            items_seen: HashSet::new(),
            work_count: 0,
            duplicate_count: 0,
        }
    }
}

pub struct WorkQueue<W: Hash + Eq> {
    work: Mutex<WorkList<W>>,
    cvar: Condvar,
    max_len: usize,
}

impl<W: Hash + Eq + Clone> WorkQueue<W> {
    pub fn new(max_len: usize) -> Self {
        Self {
            work: Mutex::new(WorkList::new()),
            cvar: Condvar::new(),
            max_len,
        }
    }

    pub fn done(&self, new_work: Vec<W>) {
        let mut worklist = self.work.lock().unwrap();
        worklist.work_count -= 1;

        for w in new_work {
            let mut h = DefaultHasher::new();
            w.hash(&mut h);
            let w64 = h.finish();
            if worklist.items_seen.insert(w64) {
                worklist.work_items.push_back(w);
                worklist.work_count += 1;
            } else {
                worklist.duplicate_count += 1
            }
        }

        self.cvar.notify_all();
    }

    pub fn push(&self, work: W) -> bool {
        let mut worklist = self.work.lock().unwrap();
        if worklist.work_items.len() > self.max_len {
            return false;
        }
        let mut h = DefaultHasher::new();
        work.hash(&mut h);
        let w64 = h.finish();
        if worklist.items_seen.insert(w64) {
            worklist.work_items.push_back(work);
            worklist.work_count += 1;
        } else {
            worklist.duplicate_count += 1
        }
        self.cvar.notify_one();
        true
    }

    pub fn pop(&self) -> Option<W> {
        let mut worklist = self.work.lock().unwrap();
        loop {
            if let Some(work) = worklist.work_items.pop_front() {
                return Some(work);
            } else if worklist.work_count == 0 {
                return None;
            }
            worklist = self.cvar.wait(worklist).unwrap()
        }
    }

    pub fn items_processed(&self) -> usize {
        let worklist = self.work.lock().unwrap();
        worklist.items_seen.len() - worklist.work_count
    }
}

impl<W: Hash + Eq> Drop for WorkQueue<W> {
    fn drop(&mut self) {
        let worklist = self.work.lock().unwrap();
        println!(
            "work queue termination after processing {} items ({} duplicates dropped, {} left)",
            worklist.items_seen.len() - worklist.work_count,
            worklist.duplicate_count,
            worklist.work_count,
        )
    }
}
