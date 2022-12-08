use rand::{seq::SliceRandom, thread_rng};

fn unique(v1: &(String, String), v2: &(String, String)) -> bool {
    v1.0 != v2.0 && v1.0 != v2.1 && v1.1 != v2.0 && v1.1 != v2.1
}

pub fn generate_pairings(all_snake_names: &Vec<String>) -> Vec<Vec<String>> {
    // generate snake pairings so that each snake will fight every other snake exactly once
    let mut combinations: Vec<(String, String)> = vec![];
    for (i, snake_1) in all_snake_names.iter().enumerate() {
        if i < all_snake_names.len() - 1 {
            for snake_2 in &all_snake_names[i + 1..all_snake_names.len()] {
                combinations.push((snake_1.clone(), snake_2.clone()));
            }
        }
    }
    if combinations.len() % 2 != 0 {
        // We need an even number of pairings for 4 player games, add an extra pair of snakes.
        let mut extra_snakes = all_snake_names.clone();
        extra_snakes.shuffle(&mut thread_rng());
        let (s1, s2) = (extra_snakes.pop().unwrap(), extra_snakes.pop().unwrap());
        combinations.push((s1, s2));
    }

    combinations.shuffle(&mut thread_rng());
    let mut pairings: Vec<Vec<String>> = vec![];
    while !combinations.is_empty() {
        let mut tmp = vec![];
        let first = combinations.pop().unwrap();
        let mut second = combinations.pop().unwrap();
        while !combinations.is_empty() && !unique(&first, &second) {
            // Can't have two of the same snakes in the same game, so grab another pair
            tmp.push(second);
            second = combinations.pop().unwrap();
        }
        if !unique(&first, &second) {
            // Tried all remaining pairs from the bag of contestants, and still didn't find
            // a suitable match. So now we look through the games we already generated,
            // and look for one where we can make a swap without having a duplicate snake in
            // either game.
            for game in &mut pairings {
                if !game.contains(&second.0) && !game.contains(&second.1) {
                    let new_second = (game[0].clone(), game[1].clone());
                    (game[0], game[1]) = second;
                    second = new_second;
                }
            }
        }
        if !unique(&first, &second) {
            // TODO: instead of panicing, maybe we can just reshuffle and retry
            panic!("failed to generate game pairings");
        }
        for v in tmp {
            // Add back rejected candidates
            combinations.push(v);
        }
        pairings.push(vec![first.0, first.1, second.0, second.1])
    }

    pairings
}

#[test]
fn test_new_round() {
    let mut snakes = vec![];
    for i in 0..50 {
        snakes.push(format!("snake_{}", i));
    }

    let games = generate_pairings(&snakes);
    for g in &games {
        for s in g {
            print!("{} ", s);
        }
        println!();
    }
    println!("Generated {} games.", games.len());
}
