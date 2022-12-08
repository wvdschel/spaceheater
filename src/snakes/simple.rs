use crate::{
    log,
    logic::{self, search, Board, Direction, Point, Tile},
    protocol, Battlesnake,
};

#[derive(Copy, Clone)]
pub struct SimpleSnake {}

fn look_ahead(board: &mut Board, head: &Point, turns: usize) -> (Direction, usize) {
    let mut max_turns = 0;
    let mut best_dir = Direction::Down;

    if turns == 0 {
        return (best_dir, max_turns);
    }

    for dir in [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ] {
        let p = head.neighbour(dir);
        if board.get(&p).is_safe() {
            let old = board.get(&p);
            board.set(&p, Tile::Wall);
            let (_, turns) = look_ahead(board, &p, turns - 1);
            match board.get(&head) {
                Tile::Head => log!("survival time for {:?}: {}", dir, turns + 1),
                _ => (),
            }
            if turns > max_turns || (turns == max_turns && rand::random()) {
                max_turns = turns + 1;
                best_dir = dir;
            }
            board.set(&p, old);
        }
    }

    (best_dir, max_turns)
}

fn wipeout(
    board: &Board,
    snakes: Vec<protocol::Snake>,
    you: &protocol::Snake,
    p: &Point,
) -> Vec<(Direction, bool, bool)> {
    let head_to_head_points = [
        (
            Direction::Left,
            [
                p.neighbour(Direction::Left).neighbour(Direction::Left),
                p.neighbour(Direction::Left).neighbour(Direction::Up),
                p.neighbour(Direction::Left).neighbour(Direction::Down),
            ],
        ),
        (
            Direction::Right,
            [
                p.neighbour(Direction::Right).neighbour(Direction::Right),
                p.neighbour(Direction::Right).neighbour(Direction::Up),
                p.neighbour(Direction::Right).neighbour(Direction::Down),
            ],
        ),
        (
            Direction::Up,
            [
                p.neighbour(Direction::Up).neighbour(Direction::Up),
                p.neighbour(Direction::Up).neighbour(Direction::Left),
                p.neighbour(Direction::Up).neighbour(Direction::Right),
            ],
        ),
        (
            Direction::Down,
            [
                p.neighbour(Direction::Down).neighbour(Direction::Down),
                p.neighbour(Direction::Down).neighbour(Direction::Left),
                p.neighbour(Direction::Down).neighbour(Direction::Right),
            ],
        ),
    ];

    let mut res = Vec::new();
    for (dir, points) in head_to_head_points {
        let (mut die, mut kill) = (false, false);
        for h in points {
            if board.get(&h) == Tile::Head {
                for s in &snakes {
                    if s.name == you.name {
                        continue;
                    }

                    if s.head == h {
                        die = die || s.length >= you.length;
                        kill = kill || s.length <= you.length;
                        log!("head-to-head with {}: die={} kill={}", s.name, die, kill);
                    }
                }
            }
        }
        res.push((dir, die, kill))
    }

    res
}

fn search_for_food(
    board: &logic::Board,
    food: &Vec<Point>,
    head: &Point,
    hp: usize,
) -> Option<Direction> {
    log!("searching for food, head is at {}", head);
    let distances = search::calculate_distances(
        board,
        head,
        |_, p| {
            let distance = if board.get(p).is_safe() { 1 } else { 9999999 };
            (distance, p.neighbours().map(|n| n.1).into())
        },
        |_, p| board.get(&p) == Tile::Food,
    );

    for f in food {
        if let Some(distance) = distances[f.x as usize][f.y as usize] {
            log!("distance to food at {} is {}", f, distance);
            if (distance + board.width()) < hp as isize && (distance > board.width()) {
                log!(
                    "not hungry yet (distance to food is {}, hp is {})",
                    distance,
                    hp
                );
                return None;
            }
            if distance > hp as isize {
                log!(
                    "distance to closest food ({}) exceeds health ({}), ignoring food",
                    distance,
                    hp
                );
                return None;
            }

            let path = search::find_path(&distances, head, f);
            match path.first() {
                Some(v) => return Some(*v),
                None => log!("can't find route to food, fall back to basic survival"),
            };
        }
    }

    None
}

impl Battlesnake for SimpleSnake {
    fn snake_info(&self) -> protocol::SnakeInfo {
        protocol::SnakeInfo {
            apiversion: "1".to_string(),
            author: "General Error".to_string(),
            color: "#ffff00".to_string(),
            head: "silly".to_string(),
            tail: "sharp".to_string(),
            version: "106b".to_string(),
        }
    }

    fn start(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: &protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: &protocol::Request) -> Result<protocol::MoveResponse, String> {
        let food = Vec::from(req.board.food.as_slice());
        let snakes = req.board.snakes.clone();
        let mut board: logic::Board = (&req.board).into();
        log!("at {} -> {:?}", req.you.head, board.get(&req.you.head));
        log!("{}", board.to_string());

        for (direction, die, kill) in wipeout(&board, snakes, &req.you, &req.you.head) {
            let p = req.you.head.neighbour(direction);
            if board.get(&p).is_safe() {
                if die {
                    // Just mark the tile of a heads-on collision as a hazard for the time being
                    board.set(&p, Tile::Hazard(3));
                } else if kill {
                    return Ok(protocol::MoveResponse {
                        direction,
                        shout: "お前はもう死んでいる".into(),
                    });
                }
            }
        }

        if let Some(dir) = search_for_food(&board, &food, &req.you.head, req.you.health as usize) {
            if board.get(&req.you.head.neighbour(dir)).is_safe() {
                return Ok(protocol::MoveResponse {
                    direction: dir,
                    shout: "nom nom nom".to_string(),
                });
            }
        }

        let (dir, _) = look_ahead(&mut board, &req.you.head, 10);
        Ok(protocol::MoveResponse {
            direction: dir,
            shout: "".to_string(),
        })
    }
}
