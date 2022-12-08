use std::{collections::HashMap, io::Read, sync::mpsc::Sender, thread::JoinHandle};

use rouille::router;

use crate::Battlesnake;

pub struct Webserver {
    address: String,
    shutdown: Sender<()>,
    join_handle: Option<JoinHandle<()>>,
}

impl Webserver {
    pub fn new(snakes: HashMap<String, Box<dyn Battlesnake + Sync + Send>>) -> Self {
        let server = rouille::Server::new("127.0.0.1:0", move |req| handle(req, &snakes)).unwrap();
        let address = server.server_addr().to_string();
        let (join_handle, shutdown) = server.stoppable();

        Webserver {
            address,
            shutdown,
            join_handle: Some(join_handle),
        }
    }

    pub fn address(&self) -> &str {
        self.address.as_str()
    }
}

impl Drop for Webserver {
    fn drop(&mut self) {
        println!("shutting down gauntlet server");
        _ = self.shutdown.send(());
        let join_handle = self.join_handle.take();
        join_handle.unwrap().join().unwrap();
    }
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
                Err(_) => None,
            }
        }
        None => None,
    };
    let body = body.unwrap_or_default();
    router!(request,
        (GET) (/{id: String}/) => {
            match snakes.get(&id) {
                Some(snake) => rouille::Response::json(&snake.snake_info()),
                None => rouille::Response::empty_404(),
            }
        },

        (POST) (/{id: String}/start) => {
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
                            rouille::Response::text(format!("{}", e)).with_status_code(400)
                        },
                    }
                },
                None => rouille::Response::empty_404(),
            }
        },

        (POST) (/{id: String}/end) => {
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
                                    rouille::Response::json(&response)
                                },
                                Err(msg) => {
                                    rouille::Response::text(msg).with_status_code(500)
                                },
                            }
                        },
                        Err(e) => {
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
