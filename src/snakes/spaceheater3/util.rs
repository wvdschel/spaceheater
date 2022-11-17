use crate::{
    logic::{Direction, Game, Point, Snake, Tile},
    protocol::ALL_DIRECTIONS,
};

pub fn certain_death(game: &Game, snake: &Snake, p: &Point) -> bool {
    match game.board.get(p) {
        Tile::Hazard(x) | Tile::HazardWithSnake(x) | Tile::HazardWithHead(x) => {
            game.rules.hazard_damage_per_turn * x as i8 > snake.health
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

pub fn all_sensible_enemy_moves(game: &Game) -> Vec<Vec<Direction>> {
    if game.others.len() == 0 {
        return vec![vec![]];
    }

    let mut all_enemy_moves: Vec<Vec<Direction>> = vec![];

    for enemy in &game.others {
        let enemy_moves: Vec<Direction> = ALL_DIRECTIONS
            .into_iter()
            .filter(|d| {
                let mut p = enemy.head.neighbour(*d);
                game.warp(&mut p);
                !certain_death(game, enemy, &p)
            })
            .collect();

        let enemy_moves = if enemy_moves.len() == 0 {
            vec![Direction::Up]
        } else {
            enemy_moves
        };
        if all_enemy_moves.is_empty() {
            all_enemy_moves = enemy_moves.into_iter().map(|d| vec![d]).collect();
        } else {
            let other_snake_moves = all_enemy_moves;
            all_enemy_moves = Vec::with_capacity(other_snake_moves.len() * enemy_moves.len());
            for enemy_move in enemy_moves {
                for other_snake_combo in &other_snake_moves {
                    let mut combo = other_snake_combo.clone();
                    combo.push(enemy_move);
                    all_enemy_moves.push(combo);
                }
            }
        }
    }

    all_enemy_moves
}
