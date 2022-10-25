use std::sync::{Condvar, Mutex};

use priority_queue::PriorityQueue;
use std::hash::Hash;

struct WorkList<W: Hash + Eq, P: Ord> {
    work_items: PriorityQueue<W, P>,
    work_count: usize,
    done_count: usize,
}

impl<W: Hash + Eq, P: Ord> WorkList<W, P> {
    pub fn new() -> Self {
        Self {
            work_items: PriorityQueue::new(),
            work_count: 0,
            done_count: 0,
        }
    }
}

pub struct WorkQueue<W: Hash + Eq, P: Ord> {
    work: Mutex<WorkList<W, P>>,
    cvar: Condvar,
    max_len: usize,
}

impl<W: Hash + Eq, P: Ord> WorkQueue<W, P> {
    pub fn new(max_len: usize) -> Self {
        Self {
            work: Mutex::new(WorkList::new()),
            cvar: Condvar::new(),
            max_len,
        }
    }

    pub fn done(&self) {
        let mut worklist = self.work.lock().unwrap();
        worklist.work_count -= 1;
        worklist.done_count += 1;
        self.cvar.notify_all();
    }

    pub fn push(&self, work: W, priority: P) -> bool {
        let mut worklist = self.work.lock().unwrap();
        if worklist.work_items.len() > self.max_len {
            return false;
        }
        if let None = worklist.work_items.push(work, priority) {
            worklist.work_count += 1; // Only update the counter if we're not replacing an existing entry
        }
        self.cvar.notify_one();
        true
    }

    pub fn pop(&self) -> Option<W> {
        let mut worklist = self.work.lock().unwrap();
        loop {
            if let Some((work, _priority)) = worklist.work_items.pop() {
                return Some(work);
            } else if worklist.work_count == 0 {
                return None;
            }
            worklist = self.cvar.wait(worklist).unwrap()
        }
    }

    pub fn items_processed(&self) -> usize {
        let worklist = self.work.lock().unwrap();
        worklist.done_count
    }
}

impl<W: Hash + Eq, P: Ord> Drop for WorkQueue<W, P> {
    fn drop(&mut self) {
        let worklist = self.work.lock().unwrap();
        println!(
            "work queue termination after processing {} items ({} left)",
            worklist.done_count, worklist.work_count,
        )
    }
}
