use std::{
    collections::VecDeque,
    ops::{Add, Neg},
};

use priority_queue::PriorityQueue;

use crate::protocol::{Direction, Point};

use super::{Board, Tile};

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
