use std::io::Read;

use rouille::{Request, Response, ResponseBody};

pub mod gamelogger;
pub mod gauntlet;
pub mod invert;
pub mod stackqueue;

#[macro_export]
#[cfg(feature = "logging")]
macro_rules! log {
    ( $( $x:tt )* ) => {{
        println!($( $x )*)
    }}
}

#[macro_export]
#[cfg(not(feature = "logging"))]
macro_rules! log {
    ( $( $x:tt )* ) => {{}};
}

#[cfg(not(feature = "sequential"))]
pub fn thread_count() -> usize {
    num_cpus::get()
}

#[cfg(feature = "sequential")]
pub fn thread_count() -> usize {
    1
}

pub fn read_body(request: &Request) -> Vec<u8> {
    match request.data() {
        Some(mut req_body) => {
            println!();
            let mut body_bytes = Vec::new();
            match req_body.read_to_end(&mut body_bytes) {
                Ok(_) => body_bytes,
                Err(e) => {
                    println!("failed to read request body: {:?}", e);
                    vec![]
                }
            }
        }
        None => vec![],
    }
}

pub fn dump_request(request: &Request) -> Option<Vec<u8>> {
    println!("{} {}", request.method(), request.raw_url());
    for (k, v) in request.headers() {
        println!("{}: {}", k, v)
    }
    match request.data() {
        Some(mut req_body) => {
            println!();
            let mut copy = Vec::new();
            match req_body.read_to_end(&mut copy) {
                Ok(_) => {
                    println!();
                    println!("{}", std::str::from_utf8(&copy).unwrap());
                    Some(copy)
                }
                Err(e) => {
                    println!("failed to read request body: {:?}", e);
                    None
                }
            }
        }
        None => None,
    }
}

pub fn dump_response(response: Response) -> Response {
    let mut res = Response {
        status_code: response.status_code,
        headers: Vec::new(),
        data: ResponseBody::empty(),
        upgrade: response.upgrade,
    };

    println!("{}", response.status_code);
    for (k, v) in &response.headers {
        res.headers.push((k.clone(), v.clone()));
        println!("{}: {}", k, v)
    }

    let (mut resp_body, _) = response.data.into_reader_and_size();
    let mut body = Vec::new();
    match resp_body.read_to_end(&mut body) {
        Ok(_) => {
            println!();
            println!("{}", std::str::from_utf8(&body).unwrap());
            res.data = ResponseBody::from_data(body);
        }
        Err(e) => println!("failed to read response body: {:?}", e),
    }

    res
}
