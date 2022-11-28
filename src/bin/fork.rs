use std::{env, num::ParseIntError};

const MAX_LEAVES_SERIALIZE: usize = 512;

fn main() {
    match parse_args() {
        Ok((target_leaves, enemy_count, max_depth, threads)) => {
            max_node(target_leaves, enemy_count, max_depth, threads as f32)
        }
        Err(_) => {
            println!("usage: fork <leaves_per_thread> <enemy_count> <max_depth> <thread_count>")
        }
    };
}

fn parse_args() -> Result<(usize, usize, usize, usize), ParseIntError> {
    let mut args = env::args();
    _ = args.next(); // Program name
    let u1 = args.next().unwrap_or("".to_string()).parse()?;
    let u2 = args.next().unwrap_or("".to_string()).parse()?;
    let u3 = args.next().unwrap_or("".to_string()).parse()?;
    let u4 = args.next().unwrap_or("".to_string()).parse()?;
    Ok((u1, u2, u3, u4))
}

fn max_node(leaves_per_thread: usize, enemy_count: usize, max_depth: usize, threads: f32) {
    let mut should_fork = false;
    let mut threads_per_child = threads;
    let leaves_per_child = max_leaf_nodes_min_node(enemy_count, max_depth);
    let total_leaves = 3 * leaves_per_child;

    if total_leaves <= MAX_LEAVES_SERIALIZE {
        println!(
            "serializing max step with {} depth left to go - estimated leaves = {}",
            max_depth, total_leaves
        );
        return;
    }

    if threads > 1f32 {
        let next_leaves_per_child = max_leaf_nodes_min_node(enemy_count, max_depth - 1);
        let target_leaves = (leaves_per_thread as f32 / threads).round() as usize;

        if leaves_per_child > target_leaves && next_leaves_per_child <= target_leaves {
            should_fork = true;
            threads_per_child = threads / 3 as f32;
        }
    }

    println!(
        "max step with {} depth left to go: fork={} leaves_per_child={} threads_per_child={}",
        max_depth, should_fork, leaves_per_child, threads_per_child
    );
    min_node(leaves_per_thread, enemy_count, max_depth, threads_per_child);
}

fn min_node(leaves_per_thread: usize, enemy_count: usize, max_depth: usize, threads: f32) {
    if max_depth == 1 && threads > 1f32 {
        println!(
            "last min step forking: {} child nodes, {} threads",
            max_leaf_nodes_min_node(enemy_count, 1),
            threads,
        );
        return;
    }
    if max_depth > 0 {
        max_node(leaves_per_thread, enemy_count, max_depth - 1, threads);
    }
}

fn max_leaf_nodes_max_node(other_snake_count: usize, depth: usize) -> usize {
    let v = (3 as usize).checked_pow(other_snake_count as u32 + 1);
    if v.is_none() {
        return usize::MAX;
    }
    let v = v.unwrap().checked_pow(depth as u32);
    v.unwrap_or(usize::MAX)
}

fn max_leaf_nodes_min_node(other_snake_count: usize, depth: usize) -> usize {
    max_leaf_nodes_max_node(other_snake_count, depth) / 3
}
