use std::collections::HashMap;

use crate::{
    logic::{game::GameMode, Point},
    util::stackqueue::StackQueue,
};

use super::{Game, Snake};

type NumType = u16;
const MAX_SNAKES: usize = 32;
const MAX_BOARD_DIMENSION: usize = 25;
const NO_SNAKE: NumType = NumType::MAX;

#[derive(Default, Copy, Clone)]
struct VoronoiTile {
    x: i8,
    y: i8,
    snake: NumType,
    distance: NumType,
    length: NumType,
}

fn stack_voronoi(game: &Game, max_distance: NumType) -> [usize; MAX_SNAKES] {
    let warp = game.rules.game_mode == GameMode::Wrapped;

    let mut queue = StackQueue::new();
    let mut scores: [usize; MAX_SNAKES] = [0; MAX_SNAKES];
    let mut board = [VoronoiTile {
        x: 0,
        y: 0,
        snake: NO_SNAKE,
        distance: NumType::MAX,
        length: 0,
    }; MAX_BOARD_DIMENSION * MAX_BOARD_DIMENSION];
    let (w, h) = (game.board.width(), game.board.height());

    fn tile_index(p: &Point) -> usize {
        p.x as usize + p.y as usize * MAX_BOARD_DIMENSION
    }

    queue.push_back(VoronoiTile {
        x: game.you.head.x,
        y: game.you.head.y,
        snake: 0,
        distance: 0,
        length: game.you.length as NumType,
    });
    for (i, snake) in game.others.iter().enumerate() {
        queue.push_back(VoronoiTile {
            x: snake.head.x,
            y: snake.head.y,
            snake: i as NumType + 1,
            distance: 0,
            length: snake.length as NumType,
        });
    }

    while let Some(work) = queue.pop_front() {
        let tile_idx = tile_index(&Point {
            x: work.x,
            y: work.y,
        });

        if board[tile_idx].snake != NO_SNAKE
            && board[tile_idx].snake != work.snake
            && board[tile_idx].distance == work.distance
            && board[tile_idx].length == work.length
        {
            // Draw: no snake gets this tile, unmark the current snake & decrease its score
            #[cfg(test)]
            println!(
                "Removing snake #{} from {},{}: draw with snake #{}",
                board[tile_idx].snake, work.x, work.y, work.snake
            );
            let snake = board[tile_idx].snake;
            board[tile_idx].snake = NO_SNAKE;
            scores[snake as usize] -= 1;
        } else if board[tile_idx].distance > work.distance
            || (board[tile_idx].distance == work.distance && board[tile_idx].length < work.length)
        {
            if board[tile_idx].snake != NO_SNAKE {
                let snake = board[tile_idx].snake;
                #[cfg(test)]
                println!(
                    "Removing snake #{} from {},{}: beaten by snake #{}",
                    snake, work.x, work.y, work.snake
                );
                scores[snake as usize] -= 1;
            }

            // We're first!
            #[cfg(test)]
            println!(
                "Snake #{} claims {},{}: distance {}",
                work.snake, work.x, work.y, work.distance
            );
            board[tile_idx] = work;
            scores[work.snake as usize] += 1;

            // Enqueue neighbouring tiles
            let next_distance = work.distance + 1;
            let p = Point {
                x: work.x,
                y: work.y,
            };
            for (_dir, mut next_p) in p.neighbours() {
                if warp {
                    next_p.warp(w, h);
                }
                // This next check does not allow traversing survivable hazards, unless hey have food.
                if next_p.out_of_bounds(w, h)
                    || game.board.hazard_count(&next_p) > 0
                    || game.board.is_snake(&next_p)
                    || game.board.is_head(&next_p)
                {
                    #[cfg(test)]
                    {
                        if next_p.out_of_bounds(w, h) {
                            println!(
                                "Not queueing {} for snake #{}: out of bounds",
                                next_p, work.snake
                            );
                        }
                        if game.board.hazard_count(&next_p) > 0 {
                            println!("Not queueing {} for snake #{}: hazard", next_p, work.snake);
                        }
                        if game.board.is_snake(&next_p) || game.board.is_head(&next_p) {
                            println!(
                                "Not queueing {} for snake #{}: collision: {}",
                                next_p,
                                work.snake,
                                game.board.get(&next_p)
                            );
                        }
                    }
                    continue;
                }

                let next_idx = tile_index(&next_p);
                if next_distance <= board[next_idx].distance && next_distance < max_distance {
                    #[cfg(test)]
                    println!("Queueing {} for snake #{}", next_p, work.snake);
                    queue.push_back(VoronoiTile {
                        x: next_p.x,
                        y: next_p.y,
                        snake: work.snake,
                        distance: next_distance,
                        length: work.length,
                    })
                }
            }
        }
    }

    #[cfg(test)]
    {
        for s in 0..(game.others.len() + 1) {
            println!("Snake #{}: {} tiles", s, scores[s]);
        }

        print!("   ");
        for x in 0..game.board.width() {
            print!("{:3}  ", x);
        }
        println!();
        for y in 0..game.board.height() as i8 {
            print!("{:2} ", y);
            for x in 0..game.board.width() as i8 {
                let idx = tile_index(&Point { x, y });
                print!("[{:3}]", board[idx].snake)
            }
            println!();
        }
    }

    scores
}

pub fn all<'a>(game: &'a Game) -> HashMap<&'a Snake, usize> {
    let mut res = HashMap::new();

    let scores = stack_voronoi(game, NumType::MAX);
    res.insert(&game.you, scores[0]);
    for (i, snake) in game.others.iter().enumerate() {
        res.insert(&snake, scores[i + 1]);
    }

    res
}

pub fn me(game: &Game) -> usize {
    stack_voronoi(game, NumType::MAX)[0]
}

pub fn me_range_limit(game: &Game, max_distance: NumType) -> usize {
    stack_voronoi(game, max_distance)[0]
}
