use std::{
    collections::HashSet,
    process::{Command, ExitStatus},
};

pub fn run_game(args: &Vec<String>, base_url: &str, snake_names: Vec<String>) -> Vec<String> {
    let mut cmd_args = vec![String::from("play")];
    cmd_args.append(&mut args.clone());
    for snake_name in &snake_names {
        cmd_args.push("-n".to_string());
        cmd_args.push(snake_name.clone());
        cmd_args.push("-u".to_string());
        cmd_args.push(format!("http://{}/{}/", base_url, snake_name));
    }

    println!("Running: battlesnake {}", args.join(" "));

    match Command::new("battlesnake")
        .args(cmd_args.as_slice())
        .output()
    {
        Ok(res) => {
            if !ExitStatus::success(&res.status) {
                println!("{}", String::from_utf8_lossy(&res.stdout));
                println!("{}", String::from_utf8_lossy(&res.stderr));
                panic!(
                    "command failed: battlesnake {}: exit code {}",
                    args.join(" "),
                    res.status
                );
            }
            let output = String::from_utf8_lossy(&res.stderr);
            let mut deaths = vec![];
            let mut alive: HashSet<String> = HashSet::from_iter(snake_names.into_iter());
            for line in output.split("\n") {
                if let Some(p) = line.find("Snakes Alive: [") {
                    let p = p + "Snakes Alive: [".len();
                    let strlen = line[p..].find("],").unwrap_or(0);
                    let mut newly_dead = alive.clone();
                    for snake in line[p..p + strlen].split(", ").map(|s| s.to_string()) {
                        newly_dead.remove(&snake);
                    }
                    for snake in newly_dead {
                        println!("{} has died", snake);
                        alive.remove(&snake);
                        deaths.push(snake);
                    }
                }
            }

            deaths.reverse();
            return deaths;
        }
        Err(error) => panic!("command failed: battlesnake {}: {}", args.join(" "), error),
    }
}
