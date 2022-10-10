#[macro_use]
extern crate rouille;
extern crate html_escape;
extern crate urlencoding;

use std::collections::HashMap;

mod logic;
mod protocol;
mod snakes;
mod util;

const DEFAULT_HOST: &str = "127.0.0.1";

pub trait Battlesnake {
    fn snake_info(&self) -> protocol::SnakeInfo;
    fn start(&self, req: protocol::Request) -> Result<(), String>;
    fn end(&self, req: protocol::Request) -> Result<(), String>;
    fn make_move(&self, req: &protocol::Request) -> Result<protocol::MoveResponse, String>;
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).map_or(DEFAULT_HOST, |v| v.as_str());
    let address = format!("{}:5110", host);

    let mut snakes = HashMap::<String, Box<dyn Battlesnake + Sync + Send>>::new();
    snakes.insert("simple".to_string(), Box::new(snakes::SimpleSnake {}));
    snakes.insert("solid".to_string(), Box::new(snakes::SolidSnake {}));
    snakes.insert(
        "spaceheater".to_string(),
        Box::new(snakes::SpaceHeater::new()),
    );

    println!("starting server on {}", address);
    rouille::start_server(address, move |request| {
        let body = util::dump_request(request).unwrap_or_default();
        let resp = router!(request,
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
                                match snake.start(request_body) {
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
                                match snake.end(request_body) {
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
                                    Ok(response) => rouille::Response::json(&response),
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

            _ => rouille::Response::empty_404()
        );
        util::dump_response(resp)
    });
}
