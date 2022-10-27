use std::collections::{HashMap, VecDeque};

use crate::{log, logic::Point};

use super::{Game, Snake};

const MAX_QUEUE_LEN: usize = 64;

struct StackQueue<T>
where
    T: Default + Copy,
{
    data: [T; MAX_QUEUE_LEN],
    head: usize,
    tail: usize,
    count: usize,
}

impl<T> StackQueue<T>
where
    T: Default + Copy,
{
    fn new() -> Self {
        Self {
            data: [T::default(); MAX_QUEUE_LEN],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let idx = self.head;
            self.head += 1;
            if self.head == self.data.len() {
                self.head = 0;
            }
            self.count -= 1;

            let res = unsafe { self.data.get_unchecked(idx) };
            Some(*res)
        }
    }

    fn push_back(&mut self, v: T) -> Result<(), ()> {
        if self.count == self.data.len() {
            Err(())
        } else {
            let slot = unsafe { self.data.get_unchecked_mut(self.tail) };
            *slot = v;
            self.tail += 1;
            if self.tail == self.data.len() {
                self.tail = 0;
            }
            self.count += 1;

            Ok(())
        }
    }
}

pub fn me(game: &Game) -> usize {
    type NumType = u8;
    #[derive(Clone, Copy)]
    struct VoronoiTile {
        distance: NumType,
        snake: NumType,
    }

    #[derive(Clone, Copy, Default)]
    struct NextTile {
        x: i8,
        y: i8,
        distance: NumType,
        snake: NumType,
    }

    const MAX_DIM: usize = 25;
    const MAX_SNAKES: usize = if (NumType::MAX - 1) > 20 {
        20
    } else {
        (NumType::MAX - 1) as usize
    };

    macro_rules! snake {
        ($i:expr) => {
            if ($i == 0) {
                &game.you
            } else {
                &game.others[$i - 1]
            }
        };
    }

    let mut count = 0;
    let (width, height) = (game.board.width() as usize, game.board.height() as usize);
    let mut board = [VoronoiTile {
        distance: NumType::MAX,
        snake: NumType::MAX,
    }; MAX_DIM * MAX_DIM];
    if game.others.len() > MAX_SNAKES {
        panic!(
            "this voronoi implementation does not support more than {} snakes",
            MAX_SNAKES
        );
    }
    if width > MAX_DIM || height > MAX_DIM {
        panic!(
            "this voronoi implementation does not support boards over {}x{}",
            MAX_DIM, MAX_DIM,
        )
    }

    let mut queue = StackQueue::new(); // VecDeque::with_capacity(MAX_QUEUE_LEN);
    queue
        .push_back(NextTile {
            x: game.you.head.x,
            y: game.you.head.y,
            distance: 0,
            snake: 0,
        })
        .unwrap();
    for (i, s) in game.others.iter().enumerate() {
        queue
            .push_back(NextTile {
                x: s.head.x,
                y: s.head.y,
                distance: 0,
                snake: i as NumType + 1,
            })
            .unwrap();
    }

    while let Some(work) = queue.pop_front() {
        let p_idx = work.x as usize + width * work.y as usize;
        let mut first = false;
        let tile = unsafe { board.get_unchecked_mut(p_idx) };
        if tile.distance > work.distance {
            first = true;
        } else if tile.distance == work.distance {
            let other = snake!(tile.snake as usize);
            let me = snake!(work.snake as usize);
            if me.length > other.length {
                first = true;
            }
        }
        if first {
            if work.snake == 0 {
                count += 1;
            }
            tile.distance = work.distance;
            tile.snake = work.snake;
            let p = Point {
                x: work.x,
                y: work.y,
            };
            if work.distance < NumType::MAX - 1 {
                for (_, p) in p.neighbours() {
                    let p = game.warp(&p);
                    let p_idx = p.x as usize + width * p.y as usize;
                    let tile = unsafe { board.get_unchecked_mut(p_idx) };
                    if tile.distance > work.distance && game.board.get(&p).is_safe() {
                        // TODO: this ignores survivable hazards
                        queue
                            .push_back(NextTile {
                                x: p.x,
                                y: p.y,
                                distance: work.distance + 1,
                                snake: work.snake,
                            })
                            .unwrap();
                    }
                }
            } else {
                log!(
                    "discarding new work after {:?} because distance counter is saturated",
                    work
                );
            }
        }
    }

    // board.iter().fold(
    //     0,
    //     |count, tile| if tile.snake == 0 { count + 1 } else { count },
    // )
    count
}

pub fn all<'a>(game: &'a Game) -> HashMap<&'a Snake, usize> {
    type NumType = u8;
    let max_snakes = (NumType::MAX - 1) as usize;
    if game.others.len() > max_snakes {
        panic!(
            "this voronoi implementation does not support more than {} snakes",
            max_snakes
        );
    }

    let mut snakes = vec![&game.you];
    for s in game.others.iter() {
        snakes.push(s);
    }
    let snakes = snakes.as_slice();

    #[derive(Clone)]
    struct VoronoiTile {
        distance: NumType,
        snake: NumType,
    }

    #[derive(Clone)]
    struct NextTile {
        point: Point,
        distance: NumType,
        snake: NumType,
    }

    let (width, height) = (game.board.width() as usize, game.board.height() as usize);
    let mut board = Vec::with_capacity(width * height);
    board.resize(
        width * height,
        VoronoiTile {
            distance: NumType::MAX,
            snake: NumType::MAX,
        },
    );
    let board = board.as_mut_slice();
    let mut counts = Vec::with_capacity(snakes.len());
    counts.resize(snakes.len(), 0);

    let mut queue = VecDeque::with_capacity(MAX_QUEUE_LEN);
    for (i, &s) in snakes.iter().enumerate() {
        queue.push_back(NextTile {
            point: s.head.clone(),
            distance: 0,
            snake: i as NumType,
        })
    }

    while let Some(work) = queue.pop_front() {
        let p_idx = work.point.x as usize + width * work.point.y as usize;
        let mut first = false;
        if board[p_idx].distance > work.distance {
            first = true;
        } else if board[p_idx].distance == work.distance {
            let other = snakes[board[p_idx].snake as usize];
            let me = snakes[work.snake as usize];
            if me.length > other.length {
                first = true;
            }
        }
        if first {
            counts[work.snake as usize] += 1;
            board[p_idx].distance = work.distance;
            board[p_idx].snake = work.snake;
            for (_, p) in work.point.neighbours() {
                let p = game.warp(&p);
                let p_idx = p.x as usize + width * p.y as usize;
                if board[p_idx].distance > work.distance && game.board.get(&p).is_safe() {
                    // TODO: this ignores survivable hazards
                    queue.push_back(NextTile {
                        point: p,
                        distance: work.distance + 1,
                        snake: work.snake,
                    })
                }
            }
        }
    }

    // let mut counts = Vec::with_capacity(snakes.len());
    // counts.resize(snakes.len(), 0);
    // for tile in board.iter() {
    //     if tile.snake == NumType::MAX {
    //         continue;
    //     }
    //     counts[tile.snake as usize] += 1;
    // }

    HashMap::from_iter(snakes.iter().enumerate().map(|(i, &s)| (s, counts[i])))
}
