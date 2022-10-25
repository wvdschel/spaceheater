use std::hash::Hash;
use std::sync::Arc;

use super::{board::BoardOverlay, Board, BoardLike, Direction, Point, Snake, Tile};
use crate::protocol;

pub struct Game {
    pub board: Arc<dyn BoardLike + Send + Sync>,
    pub others: Vec<Snake>,
    pub dead_snakes: usize,
    pub you: Snake,
    pub timeout: std::time::Duration,
    pub rules: Arc<protocol::Ruleset>,
    pub turn: usize,
}

impl Eq for Game {}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        format!("{}", self) == format!("{}", other)
    }
}

impl Hash for Game {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(format!("{}", self).as_bytes());
    }
}

pub enum GameMode {
    Standard,
    Constrictor,
    Wrapped,
    Royale,
}

impl Game {
    pub fn contains_snake(&self, name: &str) -> bool {
        if self.you.name == name && self.you.health > 0 {
            return true;
        }
        for s in &self.others {
            if s.name == name && s.health > 0 {
                return true;
            }
        }
        false
    }

    pub fn warp(&self, p: &Point) -> Point {
        if self.rules.warped_mode() {
            p.warp(self.board.width(), self.board.height())
        } else {
            p.clone()
        }
    }

    // Current implementation does not take into account:
    // - moving or expanding hazards
    // - spawning food
    pub fn execute_moves(&mut self, you: Direction, others: &Vec<Direction>) {
        let mut new_board = BoardOverlay::new(self.board.clone());
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

        self.board = Arc::new(new_board);
        self.turn += 1;
    }

    fn draw_heads(&self, board: &mut dyn BoardLike) {
        if self.you.health > 0 {
            board.add(&self.you.head, Tile::Head);
        }
        for snake in &self.others {
            if snake.health > 0 {
                board.add(&snake.head, Tile::Head);
            }
        }
    }

    fn death_by_collission(&self, snake: &Snake, board: &dyn BoardLike) -> bool {
        if board.get(&snake.head) == Tile::Snake {
            return true;
        }

        for other in self.others.iter() {
            if other.name == snake.name {
                continue;
            }

            if other.head == snake.head {
                if snake.length <= other.length {
                    return true;
                }
            }
        }

        if snake.name != self.you.name && snake.head == self.you.head {
            if snake.length <= self.you.length {
                return true;
            }
        }

        false
    }

    fn repair_crash_sites(&self, points: &Vec<Point>, new_board: &mut dyn BoardLike) {
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

    fn eliminate_dead_snakes(&mut self, new_board: &mut dyn BoardLike) {
        let mut crash_sites = vec![];
        if self.you.dead() {
            self.you.remove_from_board(new_board);
            crash_sites.push(self.you.head.clone());
            self.dead_snakes += 1;
        }
        self.others.retain(|snake| {
            if snake.dead() {
                snake.remove_from_board(new_board);
                crash_sites.push(snake.head.clone());
                self.dead_snakes += 1;
                false
            } else {
                true
            }
        });
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

impl Clone for Game {
    fn clone(&self) -> Self {
        Self {
            board: Arc::new(BoardOverlay::new(self.board.clone())),
            others: self.others.clone(),
            dead_snakes: self.dead_snakes,
            you: self.you.clone(),
            timeout: self.timeout.clone(),
            rules: self.rules.clone(),
            turn: self.turn,
        }
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
                let p = Point { x, y };
                match self.board.get(&p) {
                    Tile::Head | Tile::HazardWithHead => {
                        f.write_fmt(format_args!("<{:2}>", self.snake_number(&p)))?;
                    }
                    Tile::Snake | Tile::HazardWithSnake => {
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
        Game {
            board: Arc::new(board),
            timeout: std::time::Duration::from_millis(req.game.timeout as u64),
            you: req.you.clone(),
            others: req
                .board
                .snakes
                .iter()
                .filter(|s| s.name != req.you.name)
                .map(|s| s.clone())
                .collect(),
            rules: Arc::new(req.game.ruleset.clone()),
            dead_snakes: 0,
            turn: req.turn,
        }
    }
}

impl protocol::Ruleset {
    pub fn warped_mode(&self) -> bool {
        match self.game_mode() {
            GameMode::Wrapped => true,
            _ => false,
        }
    }

    pub fn constrictor_mode(&self) -> bool {
        match self.game_mode() {
            GameMode::Constrictor => true,
            _ => false,
        }
    }

    pub fn game_mode(&self) -> GameMode {
        match self.name.as_str() {
            "standard" => GameMode::Standard,
            "royale" => GameMode::Royale,
            "wrapped" => GameMode::Wrapped,
            _ => GameMode::Standard,
        }
    }
}
