use std::{collections::HashMap, io::Read};

use rouille::router;

use crate::Battlesnake;

pub fn run_game(
    args: &Vec<String>,
    snakes: HashMap<String, Box<dyn Battlesnake + Sync + Send>>,
) -> Vec<String> {
    let mut snake_names = vec![];
    for (name, _) in &snakes {
        snake_names.push(name.clone());
    }

    let server = rouille::Server::new("localhost:0", move |req| handle(req, &snakes)).unwrap();
    let addr = server.server_addr().to_string();
    let (join_handle, shutdown) = server.stoppable();

    let mut args = args.clone();
    for snake_name in &snake_names {
        args.push(format!("-n\"{}\"", snake_name));
        args.push(format!("-u\"http://{}/{}/\"", addr, snake_name));
    }

    // TODO: launch battlesnake, parse output
    println!("Running: battlesnake {}", args.join(" "));

    _ = shutdown.send(());
    join_handle.join().unwrap();
    snake_names
}

fn handle(
    request: &rouille::Request,
    snakes: &HashMap<String, Box<dyn Battlesnake + Sync + Send>>,
) -> rouille::Response {
    let body = match request.data() {
        Some(mut req_body) => {
            let mut copy = Vec::new();
            match req_body.read_to_end(&mut copy) {
                Ok(_) => Some(copy),
                Err(e) => {
                    println!("failed to read request body: {:?}", e);
                    None
                }
            }
        }
        None => None,
    };
    let body = body.unwrap_or_default();
    router!(request,
        (GET) (/{id: String}/) => {
            println!("request for snake info: '{}'", id);
            match snakes.get(&id) {
                Some(snake) => rouille::Response::json(&snake.snake_info()),
                None => rouille::Response::empty_404(),
            }
        },

        (POST) (/{id: String}/start) => {
            println!("starting new game for: '{}'", id);
            match snakes.get(&id) {
                Some(snake) => {
                    match serde_json::from_slice(&body) {
                        Ok(request_body) => {
                            match snake.start(&request_body) {
                                Ok(_) => rouille::Response::text(""),
                                Err(msg) => rouille::Response::text(msg).with_status_code(500),
                            }
                        },
                        Err(e) => {
                            println!("{:?}", e);
                            rouille::Response::text(format!("{}", e)).with_status_code(400)
                        },
                    }
                },
                None => rouille::Response::empty_404(),
            }
        },

        (POST) (/{id: String}/end) => {
            println!("game over for: '{}'", id);
            match snakes.get(&id) {
                Some(snake) => {
                    match serde_json::from_slice(&body) {
                        Ok(request_body) => {
                            match snake.end(&request_body) {
                                Ok(_) => rouille::Response::text(""),
                                Err(msg) => rouille::Response::text(msg).with_status_code(500),
                            }
                        },
                        Err(e) => {
                            println!("{:?}", e);
                            rouille::Response::text(format!("{}", e)).with_status_code(400)
                        },
                    }
                },
                None => rouille::Response::empty_404(),
            }
        },

        (POST) (/{id: String}/move) => {
            println!("new move for: '{}'", id);
            match snakes.get(&id) {
                Some(snake) => {
                    match serde_json::from_slice(&body) {
                        Ok(request_body) => {
                            match snake.make_move(&request_body) {
                                Ok(response) => {
                                    rouille::Response::json(&response)
                                },
                                Err(msg) => {
                                    rouille::Response::text(msg).with_status_code(500)
                                },
                            }
                        },
                        Err(e) => {
                            println!("{:?}", e);
                            rouille::Response::text(format!("{}", e)).with_status_code(400)
                        },
                    }
                },
                None => rouille::Response::empty_404(),
            }
        },

        _ => rouille::Response::empty_404()
    )
}
