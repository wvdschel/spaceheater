#[macro_use]
extern crate rouille;
extern crate html_escape;
extern crate urlencoding;

use std::collections::HashMap;

mod gamedata;
mod gamerules;
mod snakes;

pub trait Battlesnake {
    fn snake_info(&self) -> gamedata::SnakeInfo;
    fn start(&self, req: gamedata::Request) -> Result<(), String>;
    fn end(&self, req: gamedata::Request) -> Result<(), String>;
    fn make_move(&self, req: gamedata::Request) -> Result<gamedata::MoveResponse, String>;
}

fn main() {
    let snakes = HashMap::from([("simple".to_string(), Box::new(snakes::SimpleSnake {}))]);

    rouille::start_server("localhost:5110", move |request| {
        router!(request,
            (GET) (/) => {
                // List all registered snake bots
                let mut list = vec!["<html><head><title>Battle snakes</title></head><body><ul>".to_string()];
                for (name, _) in snakes.iter() {
                    list.push(
                        format!("<li><a href=\"/{}/\">{}</a></li>",
                                urlencoding::encode(name), html_escape::encode_text(name)))
                }
                list.push("</ul></body></html>".to_string());
                rouille::Response::html(list.join(""))
            },

            (GET) (/{id: String}) => {
                rouille::Response::redirect_302(format!("/{}/", urlencoding::encode(id.as_str())))
            },

            (GET) (/{id: String}/) => {
                match snakes.get(&id) {
                    Some(snake) => rouille::Response::json(&snake.snake_info()),
                    None => rouille::Response::empty_404(),
                }
            },

            (POST) (/{id: String}/start) => {
                match snakes.get(&id) {
                    Some(snake) => {
                        let body: gamedata::Request = try_or_400!(rouille::input::json_input(request));
                        match snake.start(body) {
                            Ok(_) => rouille::Response::text(""),
                            Err(msg) => rouille::Response::text(msg).with_status_code(500),
                        }
                    },
                    None => rouille::Response::empty_404(),
                }
            },

            (POST) (/{id: String}/end) => {
                match snakes.get(&id) {
                    Some(snake) => {
                        let body: gamedata::Request = try_or_400!(rouille::input::json_input(request));
                        match snake.end(body) {
                            Ok(_) => rouille::Response::text(""),
                            Err(msg) => rouille::Response::text(msg).with_status_code(500),
                        }
                    },
                    None => rouille::Response::empty_404(),
                }
            },

            (POST) (/{id: String}/move) => {
                match snakes.get(&id) {
                    Some(snake) => {
                        let body: gamedata::Request = try_or_400!(rouille::input::json_input(request));
                        match snake.make_move(body) {
                            Ok(res) => rouille::Response::json(&res),
                            Err(msg) => rouille::Response::text(msg).with_status_code(500),
                        }
                    },
                    None => rouille::Response::empty_404(),
                }
            },

            _ => rouille::Response::empty_404()
        )
    });
}
