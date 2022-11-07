use std::{
    collections::VecDeque,
    ops::{Add, Neg},
};

use priority_queue::PriorityQueue;

use crate::protocol::{Direction, Point};

use super::{Board, BoardLike, Game, Tile};

pub fn calculate_distances<T, C, B>(
    board: &dyn BoardLike,
    p: &Point,
    mut cost: C,
    mut bound: B,
) -> Vec<Vec<Option<T>>>
where
    B: FnMut(&Vec<Vec<Option<T>>>, &Point) -> bool,
    C: FnMut(&dyn BoardLike, &Point) -> (T, Vec<Point>),
    T: Clone + Copy + Ord + Default + Add<Output = T> + Neg<Output = T>,
{
    let mut distances = Vec::with_capacity(board.width() as usize);
    for i in 0..distances.capacity() {
        distances.push(Vec::new());
        distances[i].resize(board.height() as usize, None);
    }
    distances[p.x as usize][p.y as usize] = Some(T::default());

    let mut q = PriorityQueue::new();
    q.extend(p.neighbours().map(|(_, n)| (n, T::default())));

    loop {
        if let Some((next, neg_cost_so_far)) = q.pop() {
            let cost_so_far = -neg_cost_so_far;
            if next.x < 0 || next.y < 0 {
                continue;
            }
            let (x, y) = (next.x as usize, next.y as usize);
            if x >= distances.len() || y >= distances[x].len() {
                continue;
            }

            let (value, points) = cost(board, &next);
            distances[next.x as usize][next.y as usize] = Some(value.clone() + cost_so_far);

            if bound(&distances, &next) {
                break;
            }

            let cost_to_p = cost_so_far.clone() + value.clone();
            q.extend(
                points
                    .into_iter()
                    .filter(|p| {
                        if !(p.x >= 0 && p.y >= 0 && p.x < board.width() && p.y < board.height()) {
                            false
                        } else {
                            match distances[p.x as usize][p.y as usize] {
                                Some(d) => d > cost_to_p,
                                None => true,
                            }
                        }
                    })
                    .map(|v| (v, -cost_to_p)),
            );
        } else {
            break;
        }
    }

    distances
}

pub fn find_path<T>(
    distances: &Vec<Vec<Option<T>>>,
    start: &Point,
    target: &Point,
) -> Vec<Direction>
where
    T: Ord + Copy + std::fmt::Display,
{
    let mut path = VecDeque::<Direction>::new();
    let w = distances.len();
    let h = if w > 0 { distances[0].len() } else { 0 };

    let mut past_places = Board::new(w, h);
    let mut p = target.clone();
    let start = start.clone();
    while p != start {
        let mut best_dist = None;
        let mut best_dir = None;

        if past_places.get(&p) != Tile::Empty {
            // We've been walking in a circle - pop the path until we get back to the same place
            println!("cycle detected - back at {}", p);
            loop {
                if let Some(dir) = path.pop_front() {
                    let op = p.neighbour(dir);
                    if op == p {
                        // Succesfully unwound the cycle, continue
                        println!("reached {} again, succesfully unwound cycle", p);
                        break;
                    }
                    println!("walking back to {}", op);
                } else {
                    println!("could not find path between {} and {}: trying to unwind cycle at {} but path is empty", start, target, p);
                    return Vec::new();
                }
            }
        }
        // Mark p as visited
        past_places.set(&p, Tile::Wall);

        for (dir, np) in p.neighbours() {
            if past_places.get(&np) != Tile::Empty {
                continue; // Skip tiles we've already walked
            }

            if np.x >= 0 && np.y >= 0 {
                let (x, y) = (np.x as usize, np.y as usize);
                if x < distances.len() && y < distances[0].len() {
                    if let Some(dist) = distances[x][y] {
                        if let Some(to_beat) = best_dist {
                            if to_beat > dist {
                                best_dist = Some(dist);
                                best_dir = Some(dir);
                            }
                        } else {
                            best_dist = Some(dist);
                            best_dir = Some(dir);
                        }
                    }
                }
            }
        }

        if let Some(dir) = best_dir {
            path.push_front(dir.opposite());
            p = p.neighbour(dir);
        } else {
            break;
        }
    }

    path.into()
}

pub fn voronoi<'a>(game: &'a Game) -> usize {
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
            x: s.head.x as i8,
            y: s.head.y as i8,
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
                x: work.x as isize,
                y: work.y as isize,
            };
            for (_, mut p) in p.neighbours() {
                p = game.warp(&p);
                let p_idx = p.x as usize + width * p.y as usize;
                if p_idx < board.len() {
                    if board[p_idx].distance > work.distance && game.board.get(&p).is_safe() {
                        // TODO: this ignores survivable hazards
                        queue.push_back(NextTile {
                            x: p.x as i8,
                            y: p.y as i8,
                            distance: work.distance + 1,
                            snake: work.snake,
                        })
                    }
                }
            }
        }
    }

    count
}
