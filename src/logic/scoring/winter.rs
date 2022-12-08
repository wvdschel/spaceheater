use serde::{Deserialize, Serialize};
use std::{cmp, mem::MaybeUninit};

use rand::Rng;

use crate::{
    log,
    logic::{game::GameMode, Game, Point},
    snakes::Spaceheater3,
    util::{
        gauntlet::{GeneticConfig, RandomConfig},
        stackqueue::StackDequeue,
    },
};

use super::Scorer;

type NumType = u16;
pub const NO_SNAKE: u8 = u8::MAX;

#[derive(Clone, Debug)]
pub struct SnakeScore {
    pub food_count: NumType,
    pub tile_count: NumType,
    pub food_distance: NumType,
    pub food_at_min_distance: NumType,
    pub distance_to_collision: [NumType; MAX_SNAKES],
}

#[derive(Copy, Clone)]
struct TileInfo {
    snake_length: NumType,
    snake_distance: NumType,
    inaccessible_turns: NumType,
    damage_amount: i8,
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
const MAX_SNAKES: usize = 12;

pub fn winter_floodfill<const MAX_DISTANCE: NumType>(game: &Game) -> [SnakeScore; MAX_SNAKES] {
    let warp = game.rules.game_mode == GameMode::Wrapped;

    let mut queue: StackDequeue<Work, 256> = StackDequeue::new();
    let mut board = [[TileInfo {
        snake_length: 0,
        snake_distance: NumType::MAX,
        inaccessible_turns: 0,
        damage_amount: 0,
        snake: NO_SNAKE,
    }; MAX_HEIGHT]; MAX_WIDTH];

    let mut scores = {
        // Create an array of uninitialized values.
        let mut array: [MaybeUninit<SnakeScore>; MAX_SNAKES] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for element in array.iter_mut() {
            *element = MaybeUninit::new(SnakeScore {
                food_count: 0,
                tile_count: 0,
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
            let hazards = game.board.hazard_count(&Point {
                x: x as i8,
                y: y as i8,
            }) as i8;
            if hazards > 0 {
                board[x][y].damage_amount = game.rules.hazard_damage_per_turn * hazards;
            }
        }
    }

    for (idx, p) in game.you.body.iter().enumerate() {
        let (x, y) = (p.x as usize, p.y as usize);
        let present_for_turns = (game.you.length - idx - 1) as NumType;
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
            if game.board.is_food(&work.p) {
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
                if game.board.is_food(&work.p) {
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

            let tile = game.board.get(&work.p);
            let has_food = tile.has_food();

            scores[work.snake as usize].tile_count += 1;
            if has_food {
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

            let next_health = if has_food {
                100
            } else {
                work.health - board[x][y].damage_amount - 1
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
                let next_tile = game.board.get(&next_p);
                let next_has_food = next_tile.has_food();
                let next_work = Work {
                    snake_length: if has_food {
                        work.snake_length + 1
                    } else {
                        work.snake_length
                    },
                    snake_distance: work.snake_distance + 1,
                    p: next_p,
                    snake: work.snake,
                    health: next_health,
                };

                let damage = if !next_has_food {
                    board[x][y].damage_amount + 1
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Config<const MAX_DISTANCE: NumType> {
    pub points_per_food: i64,
    pub points_per_tile: i64,
    pub points_per_length_rank: i64,
    pub points_per_health: i64,
    pub points_per_distance_to_food: i64,
    pub food_distance_cap: NumType,
    pub points_per_kill: i64,
    pub points_per_turn_survived: i64,
    // should be balanced with points_per_length_rank: being longer should outweigh
    // the penalties of being far away from a smaller snake
    pub points_per_distance_to_smaller_enemies: i64,
    pub enemy_distance_cap: NumType,
    pub points_when_dead: i64,
    pub hungry_mode_max_health: i8,
    pub hungry_mode_food_multiplier: f64,
}

impl<const MAX_DISTANCE: NumType> RandomConfig for Config<MAX_DISTANCE> {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            points_per_food: rng.gen_range(0..30),
            points_per_tile: rng.gen_range(0..30),
            points_per_length_rank: rng.gen_range(-80..10),
            points_per_health: rng.gen_range(0..30),
            points_per_distance_to_food: rng.gen_range(-30..5),
            points_per_kill: rng.gen_range(0..1000),
            points_per_turn_survived: rng.gen_range(0..1000),
            points_per_distance_to_smaller_enemies: rng.gen_range(-30..5),
            points_when_dead: -10000000,
            hungry_mode_max_health: rng.gen_range(15..70),
            hungry_mode_food_multiplier: rng.gen_range((1.0)..(15.0)),
            food_distance_cap: rng.gen_range(3..50),
            enemy_distance_cap: rng.gen_range(3..50),
        }
    }
}

impl<const MAX_DISTANCE: NumType> GeneticConfig for Config<MAX_DISTANCE> {
    fn evolve(&self) -> Box<dyn GeneticConfig> {
        let mut rng = rand::thread_rng();

        let mut res = self.clone();
        let mul = rng.gen_range::<i64, _>(-3..3).pow(2);
        match rng.gen_range(0..12) {
            0 => res.points_per_food += mul,
            1 => res.points_per_tile += mul,
            2 => res.points_per_length_rank += 2 * mul,
            3 => res.points_per_health += mul,
            4 => res.points_per_distance_to_food += mul,
            5 => res.points_per_kill += 5 * mul,
            6 => res.points_per_turn_survived += 5 * mul,
            7 => res.points_per_distance_to_smaller_enemies += mul,
            8 => {
                res.hungry_mode_max_health =
                    cmp::min(100, cmp::max(0, res.hungry_mode_max_health + mul as i8))
            }
            9 => res.hungry_mode_food_multiplier += 0.05 * mul as f64,
            10 => res.food_distance_cap = cmp::max(1, res.food_distance_cap + mul as NumType),
            11 => res.enemy_distance_cap = cmp::max(1, res.enemy_distance_cap + mul as NumType),
            _ => unreachable!(),
        }

        Box::new(res)
    }

    fn battlesnake(&self) -> Box<dyn crate::Battlesnake + Sync + Send> {
        Box::new(Spaceheater3::new(self.clone(), None))
    }

    fn load(&mut self, cfg: &str) {
        *self = Self::try_from(cfg).unwrap();
    }
}

impl<const MAX_DISTANCE: NumType> ToString for Config<MAX_DISTANCE> {
    fn to_string(&self) -> String {
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();
        let stringified: Vec<String> = encoded.iter().map(|b| format!("{:02x}", b)).collect();
        stringified.join("")
    }
}

impl<const MAX_DISTANCE: NumType> TryFrom<&str> for Config<MAX_DISTANCE> {
    type Error = ();

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        let mut bytes = Vec::<u8>::with_capacity(v.len() / 2);
        for i in (0..v.len()).step_by(2) {
            if let Ok(b) = u8::from_str_radix(&v[i..i + 2], 16) {
                bytes.push(b);
            } else {
                return Err(());
            }
        }
        match bincode::deserialize::<Self>(bytes.as_slice()) {
            Ok(v) => Ok(v),
            Err(_) => Err(()),
        }
    }
}

#[test]
fn hex_encoded_config() {
    let cfg = Config::<{ u16::MAX }>::random();
    let cfg_str = cfg.to_string();

    println!("as string: {}", cfg_str);

    let cfg_parsed = Config::<{ u16::MAX }>::try_from(cfg_str.as_str()).unwrap();

    assert_eq!(cfg, cfg_parsed);
}

impl<const MAX_DISTANCE: NumType> Scorer for Config<MAX_DISTANCE> {
    fn score(&self, game: &Game) -> i64 {
        log!("scoring {}", game);
        log!("using config {:?}", self);
        let mut score: i64 = 0;
        score += self.points_per_kill * game.dead_snakes as i64;
        score += self.points_per_turn_survived * game.turn as i64;

        if game.you.dead() {
            score -= self.points_per_turn_survived + self.points_per_kill;
            score += self.points_when_dead;
            return score;
        }

        log!("points including surviving and killing: {}", score);

        let flood_info = winter_floodfill::<MAX_DISTANCE>(game);
        log!("flood fill: {:?}", flood_info);

        score += self.points_per_health * game.you.health as i64;
        log!("points including health: {}", score);
        score += self.points_per_tile * flood_info[0].tile_count as i64;
        log!(
            "points including tiles ({}): {}",
            flood_info[0].tile_count,
            score
        );

        let mut length_rank = 0;
        for (i, snake) in game.others.iter().enumerate() {
            if snake.length >= game.you.length {
                length_rank += 1;
            } else {
                score += self.points_per_distance_to_smaller_enemies
                    * cmp::min(
                        flood_info[0].distance_to_collision[i + 1],
                        self.enemy_distance_cap,
                    ) as i64;
                log!("points including distance to snake {}: {}", i, score);
            }
        }
        score += self.points_per_length_rank * length_rank;
        log!("points including length rank ({}): {}", length_rank, score);

        let mut food_score = self.points_per_food * flood_info[0].food_count as i64;
        log!(
            "points including food tiles ({}): {}",
            flood_info[0].food_count,
            score + food_score
        );
        food_score += self.points_per_distance_to_food
            * cmp::min(flood_info[0].food_distance, self.food_distance_cap) as i64;
        log!(
            "points including food distance ({}): {}",
            flood_info[0].food_distance,
            score + food_score
        );
        if game.you.health < self.hungry_mode_max_health {
            food_score = f64::round(self.hungry_mode_food_multiplier * food_score as f64) as i64;
            log!(
                "points including humgry multiplier ({}): {}",
                flood_info[0].food_distance,
                score + food_score
            );
        }

        score += food_score;

        log!("final score: {}", score);
        score
    }
}
