use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use rand::{thread_rng, Rng};

trait MutFirst<I> {
    fn first(&mut self) -> Option<I>;
}

trait First<I> {
    fn first(&self) -> Option<I>;
}

impl<T: Iterator<Item = I>, D: Clone, I: Deref<Target = D>> MutFirst<D> for T {
    fn first(&mut self) -> Option<D> {
        self.next().map(|v| (*v).clone())
    }
}

impl<I: Clone> First<I> for HashSet<I> {
    fn first(&self) -> Option<I> {
        self.iter().next().map(|v| v.clone())
    }
}

#[test]
fn test_first() {}

pub fn generate_pairings(
    all_snake_names: &Vec<String>,
    snakes_per_game: usize,
) -> Vec<Vec<String>> {
    if snakes_per_game == 1 {
        return Vec::from_iter(all_snake_names.iter().map(|s| vec![s.clone()]));
    }

    if snakes_per_game > all_snake_names.len() {
        println!("warning: asked to generate tournament roster with {} snake games, but only {} snakes in total", snakes_per_game, all_snake_names.len());
        return vec![all_snake_names.clone()];
    }

    // generate snake pairings so that each snake will fight every other snake at least once
    let mut combinations = HashMap::new();
    let mut work_left_for = HashSet::new();
    for snake_1 in all_snake_names {
        let mut set = HashSet::new();
        work_left_for.insert(snake_1.clone());
        for snake_2 in all_snake_names {
            set.insert(snake_2.clone());
        }
        combinations.insert(snake_1.clone(), set);
    }
    let mut rng = thread_rng();

    let mut pairings: Vec<Vec<String>> = vec![];

    while !work_left_for.is_empty() {
        let mut snakes = vec![work_left_for.first().unwrap()];

        while snakes.len() < snakes_per_game {
            let mut next_snake = "".to_string();
            let mut pair_count = 0;
            for s in &work_left_for {
                if snakes.contains(s) {
                    continue;
                }
                let combinations_left_for_s = combinations.get(s).unwrap();
                let count_for_s = snakes
                    .iter()
                    .filter(|s| combinations_left_for_s.contains(*s))
                    .count();
                if count_for_s > pair_count {
                    pair_count = count_for_s;
                    next_snake = s.clone();
                }
            }
            if pair_count == 0 {
                println!(
                    "couldn't find a snake to fit with {}, picking a random snake from the list",
                    snakes.join(" & "),
                );
                loop {
                    let i = rng.gen_range(0..all_snake_names.len());
                    next_snake = all_snake_names[i].clone();
                    if !snakes.contains(&next_snake) {
                        break;
                    }
                }
            }
            if next_snake == "" {
                panic!("no snake found to match with {}", snakes.join(" & "));
            }
            snakes.push(next_snake);
        }

        for s1 in &snakes {
            for s2 in &snakes {
                combinations.get_mut(s1).unwrap().remove(s2);
            }
            if combinations.get(s1).unwrap().len() == 0 {
                work_left_for.remove(s1);
            }
        }
        pairings.push(snakes);
    }

    pairings
}

#[test]
fn test_new_round() {
    let mut snakes = vec![];
    for i in 0..50 {
        snakes.push(format!("snake_{}", i));
    }

    let games = generate_pairings(&snakes, 4);
    for g in &games {
        for s in g {
            print!("{} ", s);
        }
        println!();
    }
    println!("Generated {} games.", games.len());
}
