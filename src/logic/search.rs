use std::{ops::{Add, Neg}};

use priority_queue::PriorityQueue;

use crate::protocol::Point;

use super::BoardLike;

pub fn search<T, C, B>(board: &dyn BoardLike, p: &Point, cost: C, bound: B) -> Vec<Vec<Option<T>>>
where
    B: Fn(&Vec<Vec<Option<T>>>, &Point) -> bool,
    C: Fn(&dyn BoardLike, &Point) -> (T, Vec<Point>),
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
                    .map(|v| {
                        (v, -cost_to_p)
                    }),
            );
        } else {
            break;
        }
    }

    distances
}
