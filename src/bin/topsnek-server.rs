#[macro_use]
extern crate rouille;

use std::sync::Mutex;

use topsnek::{util::gamelogger, *};

const DEFAULT_HOST: &str = "127.0.0.1";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).map_or(DEFAULT_HOST, |v| v.as_str());
    let address = format!("{}:5110", host);
    let gamelogger = Mutex::new(gamelogger::GameLogger::new());
    let snakes = snakes::snakes();

    println!("starting server on {}", address);
    rouille::start_server(address, move |request| {
        let body = util::read_body(request);
        router!(request,
            (GET) (/) => {
                // List all registered snake bots
                let mut list = vec!["<html><head><title>Battle snakes</title></head><body><ul>".to_string()];
                for (name, _) in snakes.iter() {
                    list.push(
                        format!("<li><a href=\"./{}/\">{}</a></li>",
                                urlencoding::encode(name), html_escape::encode_text(name)))
                }
                list.push("</ul></body></html>".to_string());
                rouille::Response::html(list.join(""))
            },

            (GET) (/{id: String}) => {
                rouille::Response::redirect_302(format!("/{}/", urlencoding::encode(id.as_str())))
            },

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
                                {
                                    let mut gamelogger = gamelogger.lock().unwrap();
                                    gamelogger.new_game(&request_body);
                                }
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
                                {
                                    let mut gamelogger = gamelogger.lock().unwrap();
                                    gamelogger.end_game(&request_body);
                                }
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
                match snakes.get(&id) {
                    Some(snake) => {
                        match serde_json::from_slice(&body) {
                            Ok(request_body) => {
                                match snake.make_move(&request_body) {
                                    Ok(response) => {
                                        {
                                            let mut gamelogger = gamelogger.lock().unwrap();
                                            gamelogger.log_move(&request_body, Some(&response));
                                        }
                                        println!("{}\n{} {}", logic::Game::from(&request_body), response.direction, response.shout);
                                        rouille::Response::json(&response)
                                    },
                                    Err(msg) => {
                                        {
                                            let mut gamelogger = gamelogger.lock().unwrap();
                                            gamelogger.log_move(&request_body, None);
                                        }
                                        println!("{}\nERROR {}", logic::Game::from(&request_body), msg);
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
    });
}
