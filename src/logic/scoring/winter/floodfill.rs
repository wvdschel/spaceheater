use std::{cmp, mem::MaybeUninit};

use crate::{
    logic::{game::GameMode, Game, Point, Tile},
    util::stackqueue::StackDequeue,
};

use super::{NumType, SnakeScore, MAX_SNAKES, NO_SNAKE};

struct SnakeScorePacked {
    food_counts: u64,
    score: u64,
    distance_to_collision: [NumType; MAX_SNAKES],
}

impl SnakeScorePacked {
    #[inline(always)]
    fn has_food(v: u64) -> bool {
        v & 1 == 1
    }

    #[inline(always)]
    fn hazard_count(v: u64) -> i8 {
        ((v & (0xff << 16)) >> 16) as i8
    }

    #[inline(always)]
    fn tiles(v: u64) -> u16 {
        (v >> 32) as u16
    }

    #[inline(always)]
    fn score(food: bool, hazards: u8) -> u64 {
        let food: u64 = if food { 1 } else { 0 };
        let hazards: u64 = (hazards as u64) << 16;
        (1 << 32) | hazards | food
    }

    #[inline(always)]
    fn food_count_at_distance(v: u64, distance: u8) -> u8 {
        (v >> (distance * 4)) as u8 & 0xf
    }

    #[inline(always)]
    fn food_at_distance(distance: u8) -> u64 {
        (distance as u64) << (distance * 4)
    }
}

#[derive(Copy, Clone)]
struct TileInfo {
    snake_length: NumType,
    snake_distance: NumType,
    inaccessible_turns: NumType,
    food: bool,
    hazard_count: i8,
    snake: u8,
}

#[derive(Default, Copy, Clone)]
struct Work {
    snake_length: NumType,
    snake_distance: NumType,
    p: Point,
    snake: u8,
    health: i8,
}

const MAX_WIDTH: usize = 25;
const MAX_HEIGHT: usize = 25;

pub fn floodfill<const MAX_DISTANCE: NumType>(game: &Game) -> [SnakeScore; MAX_SNAKES] {
    let constrictor = game.rules.game_mode == GameMode::Constrictor;
    let warp = game.rules.game_mode == GameMode::Wrapped;
    let hazard_damage = game.rules.hazard_damage_per_turn;

    let mut queue: StackDequeue<Work, 256> = StackDequeue::new();
    let mut board = [[TileInfo {
        snake_length: 0,
        snake_distance: NumType::MAX,
        inaccessible_turns: 0,
        hazard_count: 0,
        snake: NO_SNAKE,
        food: false,
    }; MAX_HEIGHT]; MAX_WIDTH];

    let mut scores = {
        // Create an array of uninitialized values.
        let mut array: [MaybeUninit<SnakeScore>; MAX_SNAKES] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for element in array.iter_mut() {
            *element = MaybeUninit::new(SnakeScore {
                food_count: 0,
                tile_count: 0,
                hazard_count: 0,
                food_distance: NumType::MAX,
                food_at_min_distance: 0,
                distance_to_collision: [NumType::MAX; MAX_SNAKES],
            });
        }

        unsafe { std::mem::transmute::<_, [SnakeScore; MAX_SNAKES]>(array) }
    };
    for i in 0..scores.len() {
        scores[i].distance_to_collision[i] = 0;
    }

    let (w, h) = (game.board.width() as usize, game.board.height() as usize);

    for x in 0..w {
        for y in 0..h {
            let hazards = match game.board.get(&Point {
                x: x as i8,
                y: y as i8,
            }) {
                Tile::Food => {
                    board[x][y].food = true;
                    0
                }
                Tile::Hazard(hazards)
                | Tile::HazardWithSnake(hazards)
                | Tile::HazardWithHead(hazards) => hazards,
                Tile::HazardWithFood(hazards) => {
                    board[x][y].food = true;
                    hazards
                }
                _ => 0,
            };

            board[x][y].hazard_count = hazards as i8;
        }
    }

    for (idx, p) in game.you.body.iter().enumerate() {
        let (x, y) = (p.x as usize, p.y as usize);
        let present_for_turns = if constrictor {
            NumType::MAX
        } else {
            (game.you.length - idx - 1) as NumType
        };
        board[x][y].inaccessible_turns =
            cmp::max(board[x][y].inaccessible_turns, present_for_turns);
    }

    for snake in &game.others {
        for (idx, p) in snake.body.iter().enumerate() {
            let (x, y) = (p.x as usize, p.y as usize);
            let present_for_turns = (snake.length - idx - 1) as NumType;
            board[x][y].inaccessible_turns =
                cmp::max(board[x][y].inaccessible_turns, present_for_turns);
        }
    }

    queue.push_back(Work {
        snake_length: cmp::min(game.you.length, NumType::MAX as usize) as NumType,
        snake_distance: 0,
        p: game.you.head,
        snake: 0,
        health: game.you.health,
    });
    for (i, snake) in game.others.iter().enumerate() {
        queue.push_back(Work {
            snake_length: cmp::min(snake.length, NumType::MAX as usize) as NumType,
            snake_distance: 0,
            p: snake.head,
            snake: 1 + i as u8,
            health: snake.health,
        });
    }

    while let Some(work) = queue.pop_front() {
        let (x, y) = (work.p.x as usize, work.p.y as usize);

        if board[x][y].snake != NO_SNAKE
            && board[x][y].snake != work.snake
            && board[x][y].snake_distance == work.snake_distance
            && board[x][y].snake_length == work.snake_length
        {
            // Draw: no snake gets this tile, unmark the current snake & decrease its score
            let snake = board[x][y].snake as usize;
            let work_snake = work.snake as usize;
            if scores[work_snake].distance_to_collision[snake] > work.snake_distance {
                scores[work_snake].distance_to_collision[snake] = work.snake_distance;
                scores[snake].distance_to_collision[work_snake] = work.snake_distance;
            }

            scores[snake].tile_count -= 1;
            scores[snake].hazard_count -= board[x][y].hazard_count as NumType;
            if board[x][y].food {
                scores[snake].food_count -= 1;
                if scores[snake].food_distance == work.snake_distance {
                    scores[snake].food_at_min_distance -= 1;
                    if scores[snake].food_at_min_distance == 0 {
                        scores[snake].food_distance = NumType::MAX;
                    }
                }
            }
            board[x][y].snake = NO_SNAKE;
        } else if board[x][y].snake_distance > work.snake_distance
            || (board[x][y].snake_distance == work.snake_distance
                && board[x][y].snake_length < work.snake_length)
        {
            // We're first!
            if board[x][y].snake != NO_SNAKE {
                // Remove score from previous snake if there is one
                let snake = board[x][y].snake as usize;
                scores[snake].tile_count -= 1;
                scores[snake].hazard_count -= board[x][y].hazard_count as NumType;
                if board[x][y].food {
                    scores[snake].food_count -= 1;
                    if scores[snake].food_distance == work.snake_distance {
                        scores[snake].food_at_min_distance -= 1;
                        if scores[snake].food_at_min_distance == 0 {
                            scores[snake].food_distance = NumType::MAX;
                        }
                    }
                }
                board[x][y].snake = NO_SNAKE;
            }

            scores[work.snake as usize].tile_count += 1;
            scores[work.snake as usize].hazard_count += board[x][y].hazard_count as NumType;
            if board[x][y].food {
                scores[work.snake as usize].food_count += 1;
                if scores[work.snake as usize].food_distance > work.snake_distance {
                    scores[work.snake as usize].food_distance = work.snake_distance;
                    scores[work.snake as usize].food_at_min_distance = 1;
                } else if scores[work.snake as usize].food_distance > work.snake_distance {
                    scores[work.snake as usize].food_at_min_distance += 1;
                }
            }

            // Update the board
            board[x][y].snake = work.snake;
            board[x][y].snake_distance = work.snake_distance;
            board[x][y].snake_length = work.snake_length;

            let next_health = if board[x][y].food {
                100
            } else {
                work.health - board[x][y].hazard_count * hazard_damage - 1
            };

            // Enqueue neighbouring tiles
            for (_dir, mut next_p) in work.p.neighbours() {
                if warp {
                    next_p.warp(w as isize, h as isize);
                }

                if next_p.out_of_bounds(w as isize, h as isize) {
                    // snake moves off the board
                    continue;
                }

                let (x, y) = (next_p.x as usize, next_p.y as usize);
                let next_work = Work {
                    snake_length: if board[x][y].food {
                        work.snake_length + 1
                    } else {
                        work.snake_length
                    },
                    snake_distance: work.snake_distance + 1,
                    p: next_p,
                    snake: work.snake,
                    health: next_health,
                };

                let damage = if !board[x][y].food {
                    board[x][y].hazard_count * hazard_damage + 1
                } else {
                    0
                };

                if damage >= next_work.health // snake starves or is killed by hazard
                    || board[x][y].inaccessible_turns >= next_work.snake_distance // colission
                    || next_work.snake_distance >= MAX_DISTANCE
                // reached max traveling distance
                {
                    continue;
                }

                if next_work.snake_distance > board[x][y].snake_distance {
                    // someone else got there first
                    if board[x][y].snake != NO_SNAKE {
                        let snake = board[x][y].snake as usize;
                        let work_snake = work.snake as usize;
                        if scores[work_snake].distance_to_collision[snake] > work.snake_distance {
                            scores[work_snake].distance_to_collision[snake] = work.snake_distance;
                            scores[snake].distance_to_collision[work_snake] = work.snake_distance;
                        }
                    }
                    continue;
                }

                queue.push_back(next_work)
            }
        }
    }

    #[cfg(test)]
    {
        println!("{}", game);
        for y in 0..h {
            let y = h - y - 1;
            for x in 0..w {
                let tile = game.board.get(&Point {
                    x: x as i8,
                    y: y as i8,
                });
                match tile {
                    crate::logic::Tile::Head => print!("<{:03}>", board[x][y].snake),
                    _ => print!(" {:03} ", board[x][y].snake),
                }
            }
            println!();
        }
    }

    scores
}
