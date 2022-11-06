use crate::logic::{Game, Point, Snake, Tile};

pub fn certain_death(game: &Game, snake: &Snake, p: &Point, hp: i8) -> bool {
    match game.board.get(p) {
        Tile::Hazard(x) | Tile::HazardWithSnake(x) | Tile::HazardWithHead(x) => {
            game.rules.hazard_damage_per_turn * x as i8 > hp
        }
        Tile::Snake => {
            // check self collisions. you can only hit odd body parts.
            // you should ignore your tail. you should not check other snakes
            // because they might die.
            let mut self_collision = false;
            for i in (1..snake.length - 1).step_by(2) {
                if *p == snake.body[i] {
                    self_collision = true;
                    break;
                }
            }
            self_collision
        }
        Tile::Wall => true,
        // TODO model starvation?
        _ => false,
    }
}
