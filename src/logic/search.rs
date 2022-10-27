use std::{
    collections::{HashMap, VecDeque},
    ops::{Add, Neg},
};

use priority_queue::PriorityQueue;

use crate::protocol::{Direction, Point};

use super::{Board, Game, Tile};

pub fn calculate_distances<T, C, B>(
    board: &Board,
    p: &Point,
    cost: C,
    bound: B,
) -> Vec<Vec<Option<T>>>
where
    B: Fn(&Vec<Vec<Option<T>>>, &Point) -> bool,
    C: Fn(&Board, &Point) -> (T, Vec<Point>),
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
                        if !(p.x >= 0
                            && p.y >= 0
                            && p.x < board.width() as i8
                            && p.y < board.height() as i8)
                        {
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

#[allow(unused)]
pub fn voronoi(game: &Game) -> HashMap<String, usize> {
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

            for (_, next_point) in work.point.neighbours() {
                let next_point = game.warp(&next_point);

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
            .map(|(idx, count)| (all_snakes[idx].id.clone(), count)),
    );
}

pub fn voronoi_me<'a>(game: &'a Game) -> usize {
    type NumType = u16;
    let max_snakes = (NumType::MAX - 1) as usize;
    if game.others.len() > max_snakes {
        panic!(
            "this voronoi implementation does not support more than {} snakes",
            max_snakes
        );
    }

    let mut count = 0;

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

    let mut queue = VecDeque::with_capacity(snakes.len() * 3);
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
            if work.snake == 0 {
                count += 1;
            }
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

    // board.iter().fold(
    //     0,
    //     |count, tile| if tile.snake == 0 { count + 1 } else { count },
    // )
    count
}
