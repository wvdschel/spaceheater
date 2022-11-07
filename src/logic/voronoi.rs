use std::collections::{HashMap, VecDeque};

use crate::{log, logic::Point, util::stackqueue::StackQueue};

use super::{Game, Snake};

type NumType = u8;
const MAX_SNAKES: usize = (NumType::MAX - 1) as usize;

const MAX_DIM_STACK: usize = 11;
const MAX_SNAKES_STACK: usize = if (NumType::MAX - 1) > 4 {
    4
} else {
    (NumType::MAX - 1) as usize
};

#[derive(Clone, Copy)]
struct VoronoiTileAll {
    distance: NumType,
    snake: NumType,
    length: NumType,
}

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

pub fn all_stack<'a>(game: &'a Game) -> HashMap<&'a Snake, usize> {
    macro_rules! snake {
        ($i:expr) => {
            if ($i == 0) {
                &game.you
            } else {
                &game.others[$i as usize - 1]
            }
        };
    }

    let mut counts = [0 as usize; MAX_SNAKES_STACK];
    let (width, height) = (game.board.width() as usize, game.board.height() as usize);
    let mut board = [VoronoiTileAll {
        distance: NumType::MAX,
        snake: NumType::MAX,
        length: 0,
    }; MAX_DIM_STACK * MAX_DIM_STACK];

    if game.others.len() > MAX_SNAKES_STACK {
        panic!(
            "this voronoi implementation does not support more than {} snakes",
            MAX_SNAKES_STACK
        );
    }
    if width > MAX_DIM_STACK || height > MAX_DIM_STACK {
        panic!(
            "this voronoi implementation does not support boards over {}x{}",
            MAX_DIM_STACK, MAX_DIM_STACK,
        )
    }

    let mut queue = StackQueue::new();
    queue.push_back(NextTile {
        x: game.you.head.x,
        y: game.you.head.y,
        distance: 0,
        snake: 0,
    });
    for (i, s) in game.others.iter().enumerate() {
        queue.push_back(NextTile {
            x: s.head.x,
            y: s.head.y,
            distance: 0,
            snake: i as NumType + 1,
        })
    }

    while let Some(work) = queue.pop_front() {
        let p_idx = work.x as usize + width * work.y as usize;
        let mut first = false;
        let tile = unsafe { board.get_unchecked_mut(p_idx) };
        if tile.distance > work.distance {
            first = true;
        } else if tile.distance == work.distance {
            let me = snake!(work.snake);
            if me.length > tile.length as usize {
                first = true;
            }
            if tile.snake < NumType::MAX {
                // another snake claimed this tile before, undo!
                counts[tile.snake as usize] -= 1;
                tile.snake = NumType::MAX;
            }
        }
        if first {
            counts[work.snake as usize] += 1;
            tile.distance = work.distance;
            tile.snake = work.snake;
            tile.length = snake!(tile.snake).length as NumType;
            let p = Point {
                x: work.x,
                y: work.y,
            };
            if work.distance < NumType::MAX - 1 {
                for (_, mut p) in p.neighbours() {
                    game.warp(&mut p);
                    let p_idx = p.x as usize + width * p.y as usize;
                    let tile = unsafe { board.get_unchecked_mut(p_idx) };
                    if tile.distance > work.distance && game.board.damage(&p, 1) > 0 {
                        // TODO: this ignores survivable hazards
                        queue.push_back(NextTile {
                            x: p.x,
                            y: p.y,
                            distance: work.distance + 1,
                            snake: work.snake,
                        })
                    }
                }
            } else {
                log!("discarding new work after because distance counter is saturated");
            }
        }
    }

    HashMap::from_iter((0..(game.others.len() + 1)).map(|i| (snake!(i), counts[i])))
}

pub fn me_stack(game: &Game) -> usize {
    macro_rules! snake {
        ($i:expr) => {
            if ($i == 0) {
                &game.you
            } else {
                &game.others[$i as usize - 1]
            }
        };
    }

    let mut count = 0;
    let (width, height) = (game.board.width() as usize, game.board.height() as usize);
    let mut board = [VoronoiTile {
        distance: NumType::MAX,
        snake: NumType::MAX,
    }; MAX_DIM_STACK * MAX_DIM_STACK];

    if game.others.len() > MAX_SNAKES_STACK {
        panic!(
            "this voronoi implementation does not support more than {} snakes",
            MAX_SNAKES_STACK
        );
    }
    if width > MAX_DIM_STACK || height > MAX_DIM_STACK {
        panic!(
            "this voronoi implementation does not support boards over {}x{}",
            MAX_DIM_STACK, MAX_DIM_STACK,
        )
    }

    let mut queue = StackQueue::new();
    queue.push_back(NextTile {
        x: game.you.head.x,
        y: game.you.head.y,
        distance: 0,
        snake: 0,
    });
    for (i, s) in game.others.iter().enumerate() {
        queue.push_back(NextTile {
            x: s.head.x,
            y: s.head.y,
            distance: 0,
            snake: i as NumType + 1,
        })
    }

    while let Some(work) = queue.pop_front() {
        let p_idx = work.x as usize + width * work.y as usize;
        let mut first = false;
        let tile = unsafe { board.get_unchecked_mut(p_idx) };
        if tile.distance > work.distance {
            first = true;
        } else if tile.distance == work.distance {
            let other = snake!(tile.snake);
            let me = snake!(work.snake);
            if me.length > other.length {
                first = true;
            }
            if tile.snake == 0 {
                count -= 1;
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
                for (_, mut p) in p.neighbours() {
                    game.warp(&mut p);
                    let p_idx = p.x as usize + width * p.y as usize;
                    if p_idx > board.len() {
                        continue;
                    }
                    let tile = unsafe { board.get_unchecked_mut(p_idx) };
                    if tile.distance > work.distance && game.board.damage(&p, 1) > 0 {
                        // TODO: this ignores survivable hazards
                        queue.push_back(NextTile {
                            x: p.x,
                            y: p.y,
                            distance: work.distance + 1,
                            snake: work.snake,
                        })
                    }
                }
            } else {
                log!("discarding new work after because distance counter is saturated");
            }
        }
    }

    count
}

// TODO draws/edges/overwrites are not handled correctly in heap algorithms

pub fn all_heap<'a>(game: &'a Game) -> HashMap<&'a Snake, usize> {
    if game.others.len() > MAX_SNAKES {
        panic!(
            "this voronoi implementation does not support more than {} snakes",
            MAX_SNAKES
        );
    }

    let mut snakes = vec![&game.you];
    for s in game.others.iter() {
        snakes.push(s);
    }
    let snakes = snakes.as_slice();

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

    let mut queue = VecDeque::new();
    for (i, &s) in snakes.iter().enumerate() {
        queue.push_back(NextTile {
            x: s.head.x,
            y: s.head.y,
            distance: 0,
            snake: i as NumType,
        })
    }

    while let Some(work) = queue.pop_front() {
        let p_idx = work.x as usize + width * work.y as usize;
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
            let p = Point {
                x: work.x,
                y: work.y,
            };
            for (_, mut p) in p.neighbours() {
                game.warp(&mut p);
                let p_idx = p.x as usize + width * p.y as usize;
                if p_idx > board.len() {
                    continue;
                }
                if board[p_idx].distance > work.distance && game.board.get(&p).is_safe() {
                    // TODO: this ignores survivable hazards
                    queue.push_back(NextTile {
                        x: p.x,
                        y: p.y,
                        distance: work.distance + 1,
                        snake: work.snake,
                    })
                }
            }
        }
    }

    HashMap::from_iter(snakes.iter().enumerate().map(|(i, &s)| (s, counts[i])))
}

pub fn me_heap<'a>(game: &'a Game) -> usize {
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
    let mut count = 0;

    let mut queue = VecDeque::new();
    for (i, &s) in snakes.iter().enumerate() {
        queue.push_back(NextTile {
            x: s.head.x,
            y: s.head.y,
            distance: 0,
            snake: i as NumType,
        })
    }

    while let Some(work) = queue.pop_front() {
        let p_idx = work.x as usize + width * work.y as usize;
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
            if work.snake == 0 {
                count += 1
            }
            board[p_idx].distance = work.distance;
            board[p_idx].snake = work.snake;
            let p = Point {
                x: work.x,
                y: work.y,
            };
            for (_, mut p) in p.neighbours() {
                game.warp(&mut p);
                let p_idx = p.x as usize + width * p.y as usize;
                if p_idx > board.len() {
                    continue;
                }
                if board[p_idx].distance > work.distance && game.board.get(&p).is_safe() {
                    // TODO: this ignores survivable hazards
                    queue.push_back(NextTile {
                        x: p.x,
                        y: p.y,
                        distance: work.distance + 1,
                        snake: work.snake,
                    })
                }
            }
        }
    }

    count
}

pub fn old<'a>(game: &'a Game) -> HashMap<&'a Snake, usize> {
    let w = game.board.width() as usize;
    let h = game.board.height() as usize;

    let mut res_board = Vec::with_capacity(w);
    let mut len_board = Vec::with_capacity(w);

    for _ in 0..w {
        let mut col = Vec::with_capacity(h);
        col.resize(h, None);
        res_board.push(col);

        let mut col = Vec::with_capacity(h);
        col.resize(h, 0 as usize);
        len_board.push(col);
    }

    let mut all_snakes = Vec::from([&game.you]);
    for snake in game.others.iter() {
        all_snakes.push(snake);
    }
    let all_snakes = all_snakes;
    let mut counts = Vec::new();
    counts.resize(all_snakes.len(), 0 as usize);

    struct NextTileOver {
        snake: usize,
        point: Point,
        distance: usize,
    }

    let mut queue = VecDeque::new();
    for (snake_idx, snake) in all_snakes.iter().enumerate() {
        queue.push_back(NextTileOver {
            snake: snake_idx,
            point: snake.head.clone(),
            distance: 0,
        });
    }

    let mut distance_board = res_board.clone();
    while let Some(work) = queue.pop_front() {
        let (x, y) = (work.point.x as usize, work.point.y as usize);

        let cur_snake_idx = work.snake;
        let cur_snake = all_snakes[work.snake];

        let mut first = work.distance < distance_board[x][y].unwrap_or(usize::MAX);
        if work.distance == distance_board[x][y].unwrap_or(usize::MAX) {
            // Draw - longest snake wins
            let prev_snake_len = len_board[x][y];
            if cur_snake.length > prev_snake_len {
                first = true;
            }
            if let Some(prev_snake_idx) = res_board[x][y] {
                if prev_snake_idx == cur_snake_idx {
                    continue; // Already processed this tile
                }

                let prev_snake = all_snakes[prev_snake_idx];

                if prev_snake.length <= cur_snake.length {
                    counts[prev_snake_idx] -= 1;
                }
                res_board[x][y] = None
            }
        }
        if first {
            distance_board[x][y] = Some(work.distance);
            res_board[x][y] = Some(cur_snake_idx);
            len_board[x][y] = cur_snake.length;
            counts[cur_snake_idx] += 1;

            for (_, mut next_point) in work.point.neighbours() {
                game.warp(&mut next_point);

                if next_point.out_of_bounds(w as isize, h as isize)
                    || !game.board.get(&next_point).is_safe()
                // TODO is_safe() doesn't take into account survivable hazards
                {
                    continue;
                }

                let (nx, ny) = (next_point.x as usize, next_point.y as usize);

                if let Some(cur_dist) = distance_board[nx][ny] {
                    if cur_dist < work.distance + 1 {
                        continue;
                    }
                }

                queue.push_back(NextTileOver {
                    snake: work.snake,
                    point: next_point,
                    distance: work.distance + 1,
                })
            }
        }
    }

    return HashMap::from_iter(
        counts
            .into_iter()
            .enumerate()
            .map(|(idx, count)| (all_snakes[idx], count)),
    );
}
