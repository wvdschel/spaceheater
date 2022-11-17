const MAX_BASE_DEPTH: usize = 5;
const MAX_MEASURED_DEPTH: usize = 10;
const MAX_ENEMY_COUNT: usize = 12;

fn main() {
    for enemy_count in 0..MAX_ENEMY_COUNT {
        let datfilename = format!("enemycount{}.dat", enemy_count);
        // TODO generate board with enemy_count enemies
        // TODO open dat file
        // TODO write to files, not stdout
        print!("# depth reached");
        for base_depth in 1..MAX_BASE_DEPTH {
            print!("\tbase_depth={}", base_depth);
        }
        println!();
        for max_depth in 1..MAX_MEASURED_DEPTH {
            print!("{}", max_depth);
            for base_depth in 1..MAX_BASE_DEPTH {
                let ms = base_depth * 10; // TODO run tree search and measure time if max_depth >= base_depth
                print!("\t{}", ms);
            }
            println!()
        }

        // TODO open plot file
        print!("plot");
        for base_depth in 1..MAX_BASE_DEPTH {
            println!(
                " \"{}\" using 1:{} title 'basedepth={}' with lines",
                datfilename,
                base_depth + 1,
                base_depth
            );
        }
    }
}
