use std::collections::VecDeque;

use crate::{
    logic::{self, search, Board, BoardLike, Tile},
    protocol::{self, Direction, Point},
    Battlesnake,
};

#[derive(Copy, Clone)]
pub struct SimpleSnake {}

fn look_ahead(board: &mut logic::Board, head: &Point, turns: usize) -> (Direction, usize) {
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
        if board.is_safe(&p) {
            let old = board.get(&p);
            board.set(&p, Tile::Wall);
            let (_, turns) = look_ahead(board, &p, turns - 1);
            match board.get(&head) {
                Tile::Head => println!("survival time for {:?}: {}", dir, turns + 1),
                _ => (),
            }
            if turns >= max_turns {
                max_turns = turns + 1;
                best_dir = dir;
            }
            board.set(&p, old);
        }
    }

    (best_dir, max_turns)
}

fn search_for_food(
    board: &logic::Board,
    food: &Vec<Point>,
    head: &Point,
    hp: usize,
) -> Option<Direction> {
    println!("searching for food, head is at {}", head);
    let distances = search(
        board,
        head,
        |_, p| {
            let distance = if board.is_safe(p) {
                1
            } else {
                9999999
            };
            (distance, p.neighbours().map(|n| n.1).into())
        },
        |_, p| board.get(&p) == Tile::Food,
    );

    for f in food {
        if let Some(distance) = distances[f.x as usize][f.y as usize] {
            println!("distance to food at {} is {}", f, distance);
            if (distance + board.width()) < hp as isize {
                println!(
                    "not hungry yet (distance to food is {}, hp is {})",
                    distance, hp
                );
                return None;
            }
            if distance > hp as isize {
                println!("distance to closest food ({}) exceeds health ({}), ignoring food", distance, hp);
                return None;
            }
            
            let path = find_path(&distances, head, f);
            for d in path.iter() {
                print!("{} ", d);
            }
            println!();
            match path.first() {
                Some(v) => return Some(*v),
                None => println!("can't find route to food, fall back to basic survival"),
            };
        }
    }

    None
}

fn find_path<T>(distances: &Vec<Vec<Option<T>>>, start: &Point, target: &Point) -> Vec<Direction>
where
    T: Ord + Copy + std::fmt::Display,
{
    let mut path = VecDeque::<Direction>::new();
    let w = distances.len();
    let h = if w > 0 { distances[0].len() } else { 0 };

    println!("finding path between {} and {}, distance is {}", start, target, distances[target.x as usize][target.y as usize].unwrap());

    for y_ in 0..h {
        for x in 0..w {
            let y = h - y_ - 1;
            let mut marker = '|';
            if x as isize == start.x && y as isize == start.y {
                marker = '*';
            }
            if x as isize == target.x && y as isize == target.y {
                marker = ':';
            }
            match distances[x][y] {
                Some(d) => {
                    let mut distance = format!("{:03}", d);
                    if distance.len() > 3 {
                        distance = "INF".into();
                    }
                    print!("{}{}{}", marker, distance, marker);
                },
                None => {
                    print!("{}   {}", marker, marker);
                },
            }
        }
        println!();
    }

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
                println!("skipping {} - already visited", np);
                continue; // Skip tiles we've already walked
            }
            println!("evaluating going {:?} to {}", dir, np);

            if np.x >= 0 && np.y >= 0 {
                let (x, y) = (np.x as usize, np.y as usize);
                if x < distances.len() && y < distances[0].len() {
                    if let Some(dist) = distances[x][y] {
                        if let Some(to_beat) = best_dist {
                            if to_beat > dist {
                                println!("going {:?} to {} is better than {:?} (d={}), distance is now {}", 
                                dir, np, best_dir.unwrap(), to_beat, dist);
                                best_dist = Some(dist);
                                best_dir = Some(dir);
                            }
                        } else {
                            println!(
                                "going {:?} to {} is better than nothing, distance is now {}",
                                dir, np, dist
                            );
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
            println!("-> we're going {:?} to {}", dir, p);
        } else {
            println!("could not find path between {} and {}", start, target);
            break;
        }
    }

    path.into()
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

    fn start(&self, _: protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn end(&self, _: protocol::Request) -> Result<(), String> {
        Ok(())
    }

    fn make_move(&self, req: protocol::Request) -> Result<protocol::MoveResponse, String> {
        let food = Vec::from(req.board.food.as_slice());
        let mut board: logic::Board = req.board.into();
        let head = req.you.head;
        println!("at {} -> {:?}", head, board.get(&head));
        println!("{}", &board as &dyn BoardLike);

        if let Some(dir) = search_for_food(&board, &food, &head, req.you.health) {
            return Ok(protocol::MoveResponse {
                direction: dir,
                shout: "food".to_string(),
            });
        }

        let (dir, _) = look_ahead(&mut board, &head, 10);
        Ok(protocol::MoveResponse {
            direction: dir,
            shout: "".to_string(),
        })
    }
}
