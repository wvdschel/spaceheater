use std::{
    fs::{self},
    io::Write,
};

use topsnek::logic::scoring::winter;

const REFERENCE_SNAKES: [&str; 1] = ["spaceheater"];
const SNAKE_COUNT: usize = 180;
const CONFIG_DIR: &str = "./cfg";

fn main() {
    fs::create_dir_all(CONFIG_DIR).unwrap();
    let paths = fs::read_dir(CONFIG_DIR).unwrap();
    let config_count = paths.count();

    if config_count < SNAKE_COUNT {
        println!(
            "Missing snakes: only found {} out of {}",
            config_count, SNAKE_COUNT
        );
        for _ in 0..(config_count - SNAKE_COUNT) {
            let mut new_snake = winter::Config::<{ u16::MAX }>::random();
            let mut path =
                std::path::Path::from(format!("{}/{}", CONFIG_DIR, new_snake.to_string()));
            while path.exists() {
                new_snake = winter::Config::<{ u16::MAX }>::random();
                path = std::path::Path::from(format!("{}/{}", CONFIG_DIR, new_snake.to_string()));
            }

            let mut file = fs::File::create(path).unwrap();
            file.write_all(b"0").unwrap();
        }
    }

    let paths = fs::read_dir(CONFIG_DIR).unwrap();
    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }
}
