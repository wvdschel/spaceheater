use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
};

struct WorkList<T: Sized> {
    work_items: VecDeque<T>,
    work_count: usize,
}

impl<T: Sized> WorkList<T> {
    pub fn new() -> Self {
        Self {
            work_items: VecDeque::new(),
            work_count: 0,
        }
    }
}

pub struct WorkQueue<T: Sized> {
    work: Mutex<WorkList<T>>,
    cvar: Condvar,
}

impl<T: Sized> WorkQueue<T> {
    pub fn new() -> Self {
        Self {
            work: Mutex::new(WorkList::new()),
            cvar: Condvar::new(),
        }
    }

    pub fn done(&mut self) {
        let mut worklist = self.work.lock().unwrap();
        worklist.work_count -= 1;
        self.cvar.notify_all();
    }

    pub fn push(&mut self, work: T) {
        let mut worklist = self.work.lock().unwrap();
        worklist.work_count += 1;
        worklist.work_items.push_back(work);
        self.cvar.notify_one();
    }

    pub fn pop(&mut self) -> Option<T> {
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
}
