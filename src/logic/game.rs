use std::hash::Hash;

use super::{Board, Direction, Point, Snake, Tile};
use crate::protocol;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Game {
    pub board: Board,
    pub others: Vec<Snake>,
    pub dead_snakes: usize,
    pub you: Snake,
    pub timeout: std::time::Duration,
    pub rules: Rules,
    pub turn: usize,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Rules {
    pub game_mode: GameMode,
    pub hazard_damage_per_turn: i8,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameMode {
    Standard,
    Constrictor,
    Wrapped,
    Royale,
    Solo,
}

impl Game {
    pub fn warp(&self, p: &mut Point) {
        if self.rules.game_mode == GameMode::Wrapped {
            p.warp(self.board.width(), self.board.height())
        }
    }

    // Current implementation does not take into account:
    // - moving or expanding hazards
    // - spawning food
    pub fn execute_moves(&mut self, you: Direction, others: &Vec<Direction>) {
        let mut new_board = self.board.clone();
        self.you.apply_move(you, &mut new_board, &self.rules);
        for i in 0..others.len() {
            self.others[i].apply_move(others[i], &mut new_board, &self.rules)
        }

        self.eliminate_dead_snakes(&mut new_board);

        if self.death_by_collission(&self.you, &new_board) {
            self.you.health = 0;
        }

        let mut deaths = Vec::new();
        for snake in self.others.iter() {
            deaths.push(self.death_by_collission(snake, &new_board));
        }
        for i in 0..deaths.len() {
            if deaths[i] {
                self.others[i].health = 0;
            }
        }

        self.eliminate_dead_snakes(&mut new_board);
        self.draw_heads(&mut new_board);

        self.board = new_board;
        self.turn += 1;
    }

    fn draw_heads(&self, board: &mut Board) {
        if self.you.health > 0 {
            board.add(&self.you.head, Tile::Head);
        }
        for snake in &self.others {
            if snake.health > 0 {
                board.add(&snake.head, Tile::Head);
            }
        }
    }

    fn death_by_collission(&self, snake: &Snake, board: &Board) -> bool {
        match board.get(&snake.head) {
            Tile::HazardWithSnake(_) | Tile::Snake => return true,
            _ => {}
        }

        for other in self.others.iter() {
            if other == snake {
                continue;
            }

            if other.head == snake.head {
                if snake.length <= other.length {
                    return true;
                }
            }
        }

        if snake != &self.you && snake.head == self.you.head {
            if snake.length <= self.you.length {
                return true;
            }
        }

        false
    }

    fn repair_crash_sites(&self, points: &Vec<Point>, new_board: &mut Board) {
        for p in points {
            if self.you.health > 0 {
                for b in &self.you.body {
                    if b == p {
                        new_board.add(p, Tile::Snake);
                    }
                }
                if &self.you.head == p {
                    new_board.add(p, Tile::Head);
                }
            }

            for snake in &self.others {
                for b in &snake.body {
                    if b == p {
                        new_board.add(p, Tile::Snake);
                    }
                }
                if &snake.head == p {
                    new_board.add(p, Tile::Head);
                }
            }
        }
    }

    fn eliminate_dead_snakes(&mut self, new_board: &mut Board) {
        let mut dead_snakes = 0;
        let mut crash_sites = vec![];
        if self.you.dead() {
            self.you.remove_from_board(new_board);
            crash_sites.push(self.you.head.clone());
            dead_snakes += 1;
        }
        self.others.retain(|snake| {
            if snake.dead() {
                snake.remove_from_board(new_board);
                crash_sites.push(snake.head.clone());
                dead_snakes += 1;
                false
            } else {
                true
            }
        });
        self.dead_snakes = dead_snakes;
        self.repair_crash_sites(&crash_sites, new_board);
    }

    fn snake_number(&self, p: &Point) -> isize {
        if self.you.health > 0 {
            if &self.you.head == p {
                return 0;
            }
            for b in &self.you.body {
                if b == p {
                    return 0;
                }
            }
        }

        for snake_idx in 0..self.others.len() {
            let snake = &self.others[snake_idx];
            if &snake.head == p {
                return snake_idx as isize + 1;
            }
            for bp in &snake.body {
                if bp == p {
                    return snake_idx as isize + 1;
                }
            }
        }

        return -1;
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Turn {}\n", self.turn))?;
        f.write_fmt(format_args!("Snakes: ({} have died)\n", self.dead_snakes))?;
        f.write_fmt(format_args!("0: {} (you)\n", self.you))?;
        for i in 0..self.others.len() {
            f.write_fmt(format_args!("{}: {}\n", i + 1, self.others[i]))?;
        }
        for ny in 0..self.board.height() {
            let y = self.board.height() - 1 - ny;
            for x in 0..self.board.width() {
                let p = Point {
                    x: x as i8,
                    y: y as i8,
                };
                match self.board.get(&p) {
                    Tile::Head | Tile::HazardWithHead(_) => {
                        f.write_fmt(format_args!("<{:2}>", self.snake_number(&p)))?;
                    }
                    Tile::Snake | Tile::HazardWithSnake(_) => {
                        f.write_fmt(format_args!("[{:2}]", self.snake_number(&p)))?;
                    }
                    t => {
                        f.write_fmt(format_args!("  {} ", t))?;
                    }
                }
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl From<&protocol::Request> for Game {
    fn from(req: &protocol::Request) -> Self {
        let board: Board = (&req.board).into();

        let mut you = Snake::from(&req.you);
        you.id = 0;

        let others = req
            .board
            .snakes
            .iter()
            .filter(|s| s.id != req.you.id)
            .enumerate()
            .map(|(i, s)| {
                let mut s = Snake::from(s);
                s.id = i as u8 + 1;
                s
            })
            .collect();

        Game {
            board: board,
            timeout: std::time::Duration::from_millis(req.game.timeout as u64),
            you,
            others,
            rules: Rules::from(&req.game.ruleset),
            dead_snakes: 0,
            turn: req.turn,
        }
    }
}

impl From<&protocol::Ruleset> for Rules {
    fn from(r: &protocol::Ruleset) -> Self {
        Self {
            game_mode: match r.name.as_str() {
                "standard" => GameMode::Standard,
                "royale" => GameMode::Royale,
                "wrapped" => GameMode::Wrapped,
                "solo" => GameMode::Solo,
                _ => {
                    println!("unknown game mode: {}", r.name);
                    GameMode::Standard
                }
            },
            hazard_damage_per_turn: r.settings.hazard_damage_per_turn as i8,
        }
    }
}
