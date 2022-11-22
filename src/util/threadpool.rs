use std::{
    collections::VecDeque,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    task::Poll,
    thread,
};

use crate::log;

use super::thread_count;

pub struct Threadpool<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    cancelled: Arc<AtomicBool>,
    work: Arc<Mutex<VecDeque<Work<In, Out>>>>,
    work_available: Arc<Condvar>,
}

struct Work<In: Send, Out: Send> {
    work: In,
    join_channel: Sender<Out>,
}

pub struct JoinHandle<Out: Send> {
    join_channel: Receiver<Out>,
    result: Option<Out>,
}

impl<In, Out> Threadpool<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    pub fn new<F>(work_fn: F) -> Self
    where
        F: Send + Clone + FnMut(In) -> Out + 'static,
    {
        let q: VecDeque<Work<In, Out>> = VecDeque::new();
        let work = Arc::new(Mutex::new(q));
        let work_available = Arc::new(Condvar::new());
        let cancelled = Arc::new(AtomicBool::new(false));

        let tp = Self {
            cancelled,
            work,
            work_available,
        };

        for thread_i in 0..thread_count() {
            let tp = tp.clone();
            let mut work_fn = work_fn.clone();

            thread::spawn(move || {
                loop {
                    match tp.pop() {
                        Some(work_item) => {
                            let res = work_fn(work_item.work);
                            if let Err(_e) = work_item.join_channel.send(res) {
                                log!("thread {}: failed to send result: {}", thread_i, _e);
                            }
                        }
                        None => break,
                    }
                }
                println!("thread {}: returning", thread_i);
            });
        }
        tp
    }

    fn pop(&self) -> Option<Work<In, Out>> {
        let mut work = self.work.lock().unwrap();
        loop {
            match work.pop_front() {
                Some(w) => {
                    return Some(w);
                }
                None => {
                    work = self.work_available.wait(work).unwrap();
                }
            }
            if self.cancelled.load(Ordering::Relaxed) {
                return None;
            }
        }
    }

    pub fn post(&self, input: In) -> JoinHandle<Out> {
        let mut work = self.work.lock().unwrap();
        let (snd, rcv) = channel();
        work.push_back(Work {
            work: input,
            join_channel: snd,
        });

        self.work_available.notify_one();
        JoinHandle {
            join_channel: rcv,
            result: None,
        }
    }

    pub fn post_many(&self, inputs: Vec<In>) -> Vec<JoinHandle<Out>> {
        let mut work = self.work.lock().unwrap();
        let mut res = vec![];

        for input in inputs {
            let (snd, rcv) = channel();
            work.push_back(Work {
                work: input,
                join_channel: snd,
            });
            res.push(JoinHandle {
                join_channel: rcv,
                result: None,
            })
        }

        res
    }

    pub fn clear(&mut self) {
        let mut work = self.work.lock().unwrap();
        work.clear();
        self.work_available.notify_all();
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl<In: Send, Out: Send> Clone for Threadpool<In, Out> {
    fn clone(&self) -> Self {
        Self {
            cancelled: self.cancelled.clone(),
            work: self.work.clone(),
            work_available: self.work_available.clone(),
        }
    }
}

impl<In, Out> Drop for Threadpool<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    fn drop(&mut self) {
        self.cancel()
    }
}

impl<Out: Send> JoinHandle<Out> {
    pub fn is_finished(&mut self) -> bool {
        if self.result.is_some() {
            return true;
        }

        match self.join_channel.try_recv() {
            Ok(res) => {
                self.result = Some(res);
                return true;
            }
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => return false,
                std::sync::mpsc::TryRecvError::Disconnected => return true,
            },
        }
    }

    pub fn join(self) -> Out {
        if let Some(res) = self.result {
            return res;
        }

        self.join_channel.recv().unwrap()
    }
}

#[test]
fn test_threadpool() {
    let mut fibs = vec![1, 1];
    for _ in 2..50 {
        fibs.push(fibs[fibs.len() - 1] + fibs[fibs.len() - 2]);
    }

    fn fibonacci(n: usize) -> usize {
        match n {
            0 => 1,
            1 => 1,
            n => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    let threadpool = Threadpool::new(&fibonacci);

    let work_results: Vec<_> = (0..16).map(|i| threadpool.post(20 + i)).collect();

    for (n, res) in work_results.into_iter().enumerate() {
        let fib_n = res.join();
        assert_eq!(fib_n, fibs[20 + n]);
    }
}
